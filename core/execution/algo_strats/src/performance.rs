//! Strategy performance tracking: P&L, drawdown, win rate, Sharpe.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use chrono::{DateTime, Utc};

/// Performance metrics for a strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub strategy_id: uuid::Uuid,
    pub strategy_name: String,
    pub total_pnl_usd: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate_pct: f64,
    pub avg_win_usd: f64,
    pub avg_loss_usd: f64,
    pub profit_factor: f64,
    pub max_drawdown_pct: f64,
    pub current_drawdown_pct: f64,
    pub sharpe_ratio: f64,
    pub total_fees_usd: f64,
    pub total_slippage_usd: f64,
    pub uptime_pct: f64,
    pub started_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub daily_returns: VecDeque<f64>,
}

impl StrategyPerformance {
    pub fn new(strategy_id: uuid::Uuid, name: &str) -> Self {
        Self {
            strategy_id,
            strategy_name: name.to_string(),
            total_pnl_usd: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            win_rate_pct: 0.0,
            avg_win_usd: 0.0,
            avg_loss_usd: 0.0,
            profit_factor: 0.0,
            max_drawdown_pct: 0.0,
            current_drawdown_pct: 0.0,
            sharpe_ratio: 0.0,
            total_fees_usd: 0.0,
            total_slippage_usd: 0.0,
            uptime_pct: 100.0,
            started_at: Utc::now(),
            last_updated: Utc::now(),
            daily_returns: VecDeque::with_capacity(365),
        }
    }

    /// Records a completed trade.
    pub fn record_trade(&mut self, pnl_usd: f64, fees: f64, slippage: f64) {
        self.total_trades += 1;
        self.total_pnl_usd += pnl_usd;
        self.total_fees_usd += fees;
        self.total_slippage_usd += slippage;

        if pnl_usd > 0.0 {
            self.winning_trades += 1;
        } else {
            self.losing_trades += 1;
        }

        self.win_rate_pct = (self.winning_trades as f64 / self.total_trades.max(1) as f64) * 100.0;

        let total_wins: f64 = self.total_pnl_usd.max(0.0);
        let total_losses: f64 = (-self.total_pnl_usd).max(0.0);
        self.profit_factor = if total_losses > 0.0 {
            total_wins / total_losses
        } else {
            f64::INFINITY
        };

        self.last_updated = Utc::now();
    }

    /// Records a daily return for Sharpe calculation.
    pub fn record_daily_return(&mut self, daily_return_pct: f64) {
        self.daily_returns.push_back(daily_return_pct);
        if self.daily_returns.len() > 365 {
            self.daily_returns.pop_front();
        }
        self.calculate_sharpe();
    }

    fn calculate_sharpe(&mut self) {
        if self.daily_returns.len() < 2 {
            self.sharpe_ratio = 0.0;
            return;
        }
        let mean = self.daily_returns.iter().sum::<f64>() / self.daily_returns.len() as f64;
        let variance = self.daily_returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / (self.daily_returns.len() - 1) as f64;
        let std = variance.sqrt();
        self.sharpe_ratio = if std > 0.0 {
            (mean / std) * (365.0_f64).sqrt()
        } else {
            0.0
        };
    }

    /// Calculates current drawdown from peak.
    pub fn update_drawdown(&mut self, current_value: f64, peak_value: f64) {
        if peak_value > 0.0 {
            let dd = ((peak_value - current_value) / peak_value) * 100.0;
            self.current_drawdown_pct = dd.max(0.0);
            self.max_drawdown_pct = self.max_drawdown_pct.max(dd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_tracking() {
        let mut perf = StrategyPerformance::new(uuid::Uuid::new_v4(), "Test DCA");

        perf.record_trade(100.0, 2.0, 0.5);
        perf.record_trade(-50.0, 1.5, 0.3);
        perf.record_trade(200.0, 3.0, 0.8);

        assert_eq!(perf.total_trades, 3);
        assert_eq!(perf.winning_trades, 2);
        assert_eq!(perf.losing_trades, 1);
        assert!((perf.total_pnl_usd - 250.0).abs() < 0.01);
        assert!((perf.win_rate_pct - 66.67).abs() < 0.1);
        assert!(perf.profit_factor > 1.0);
    }

    #[test]
    fn test_drawdown_calculation() {
        let mut perf = StrategyPerformance::new(uuid::Uuid::new_v4(), "Test");

        perf.update_drawdown(90_000.0, 100_000.0);
        assert!((perf.current_drawdown_pct - 10.0).abs() < 0.01);
        assert!((perf.max_drawdown_pct - 10.0).abs() < 0.01);

        perf.update_drawdown(80_000.0, 100_000.0);
        assert!((perf.current_drawdown_pct - 20.0).abs() < 0.01);
        assert!((perf.max_drawdown_pct - 20.0).abs() < 0.01);
    }
}   