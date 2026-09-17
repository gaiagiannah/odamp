//! Momentum strategy.
//!
//! Buys assets showing strong upward momentum (rate of change)
//! and sells when momentum fades. Works in trending markets.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::strategy::{Strategy, StrategyConfig, StrategySignal, StrategyState, StrategyStatus, SignalAction, MarketSnapshot};
use crate::error::StrategyError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumParams {
    pub lookback_periods: u32,
    pub entry_roc_pct: f64,
    pub exit_roc_pct: f64,
    pub order_size_usd: f64,
    pub volume_confirmation: bool,
}

pub struct MomentumStrategy {
    id: uuid::Uuid,
    config: StrategyConfig,
    state: StrategyState,
    params: MomentumParams,
    price_history: VecDeque<f64>,
    volume_history: VecDeque<f64>,
    in_position: bool,
    entry_price: Option<f64>,
}

impl MomentumStrategy {
    pub fn new(config: StrategyConfig, params: MomentumParams) -> Self {
        let id = uuid::Uuid::new_v4();
        Self {
            id,
            config,
            state: StrategyState {
                id: id.clone(),
                status: StrategyStatus::Running,
                started_at: chrono::Utc::now(),
                stopped_at: None,
                total_trades: 0,
                total_pnl_usd: 0.0,
                current_drawdown_pct: 0.0,
                daily_spent_usd: 0.0,
                positions: vec![],
            },
            params,
            price_history: VecDeque::with_capacity(params.lookback_periods as usize),
            volume_history: VecDeque::with_capacity(params.lookback_periods as usize),
            in_position: false,
            entry_price: None,
        }
    }

    fn rate_of_change(&self) -> f64 {
        let n = self.price_history.len();
        if n < 2 { return 0.0; }
        let first = *self.price_history.front().unwrap();
        let last = *self.price_history.back().unwrap();
        if first == 0.0 { return 0.0; }
        ((last - first) / first) * 100.0
    }

    fn avg_volume(&self) -> f64 {
        if self.volume_history.is_empty() { return 0.0; }
        self.volume_history.iter().sum::<f64>() / self.volume_history.len() as f64
    }
}

#[async_trait]
impl Strategy for MomentumStrategy {
    fn id(&self) -> uuid::Uuid { self.id }
    fn name(&self) -> &str { "Momentum" }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn state(&self) -> &StrategyState { &self.state }

    async fn on_tick(&mut self, market: &MarketSnapshot) -> Result<Option<StrategySignal>, StrategyError> {
        if self.state.status != StrategyStatus::Running {
            return Ok(None);
        }

        let symbol = &self.config.symbols[0];
        let price = market.prices.get(symbol).copied().unwrap_or(0.0);
        let volume = market.volumes.get(symbol).copied().unwrap_or(0.0);
        if price == 0.0 {
            return Err(StrategyError::NoMarketData(self.id));
        }

        self.price_history.push_back(price);
        self.volume_history.push_back(volume);
        let lookback = self.params.lookback_periods as usize;
        if self.price_history.len() > lookback { self.price_history.pop_front(); }
        if self.volume_history.len() > lookback { self.volume_history.pop_front(); }

        if self.price_history.len() < lookback / 2 {
            return Ok(None);
        }

        let roc = self.rate_of_change();

        // Volume confirmation
        let volume_ok = if self.params.volume_confirmation {
            volume > self.avg_volume() * 1.2
        } else {
            true
        };

        // Entry: strong upward momentum + volume
        if !self.in_position && roc > self.params.entry_roc_pct && volume_ok {
            self.in_position = true;
            self.entry_price = Some(price);
            self.state.total_trades += 1;

            return Ok(Some(StrategySignal {
                strategy_id: self.id,
                symbol: symbol.clone(),
                action: SignalAction::Buy,
                amount: self.params.order_size_usd / price,
                amount_usd: self.params.order_size_usd,
                limit_price: None,
                reason: format!("Momentum: entry at RoC={:.2}% (threshold {:.2}%)", roc, self.params.entry_roc_pct),
                confidence: (roc / (self.params.entry_roc_pct * 3.0)).min(1.0),
                timestamp: market.timestamp,
            }));
        }

        // Exit: momentum fades
        if self.in_position && roc < self.params.exit_roc_pct {
            self.in_position = false;
            let entry = self.entry_price.unwrap_or(price);
            let pnl = (price - entry) * (self.params.order_size_usd / entry);
            self.state.total_pnl_usd += pnl;
            self.entry_price = None;
            self.state.total_trades += 1;

            return Ok(Some(StrategySignal {
                strategy_id: self.id,
                symbol: symbol.clone(),
                action: SignalAction::Sell,
                amount: self.params.order_size_usd / entry,
                amount_usd: price * (self.params.order_size_usd / entry),
                limit_price: None,
                reason: format!("Momentum: exit at RoC={:.2}% (P&L ${:.2})", roc, pnl),
                confidence: 0.8,
                timestamp: market.timestamp,
            }));
        }

        Ok(None)
    }

    async fn on_fill(&mut self, _signal: &StrategySignal, _filled: bool, _fill_price: Option<f64>) {}

    fn stop(&mut self, reason: &str) {
        self.state.status = StrategyStatus::Stopped;
        self.state.stopped_at = Some(chrono::Utc::now());
    }

    fn check_risk_limits(&self) -> Option<String> {
        if self.state.current_drawdown_pct > self.config.max_drawdown_pct {
            return Some("Max drawdown exceeded".into());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_momentum_entry() {
        let config = StrategyConfig {
            name: "SOL Momentum".into(),
            symbols: vec!["SOL".into()],
            initial_capital_usd: 5_000.0,
            max_position_usd: 10_000.0,
            max_daily_spend_usd: 5_000.0,
            max_drawdown_pct: 20.0,
            stop_loss_pct: 15.0,
            take_profit_pct: None,
            params: serde_json::json!({}),
        };

        let params = MomentumParams {
            lookback_periods: 20,
            entry_roc_pct: 5.0,
            exit_roc_pct: 1.0,
            order_size_usd: 500.0,
            volume_confirmation: false,
        };

        let mut strat = MomentumStrategy::new(config, params);

        // Feed rising prices
        for i in 0..20 {
            let market = MarketSnapshot {
                timestamp: chrono::Utc::now() + chrono::Duration::minutes(i),
                prices: std::collections::HashMap::from([("SOL".to_string(), 100.0 + i as f64 * 0.5)]),
                volumes: std::collections::HashMap::from([("SOL".to_string(), 1_000_000.0)]),
                funding_rates: std::collections::HashMap::new(),
                volatility: std::collections::HashMap::new(),
            };
            let _ = strat.on_tick(&market).await.unwrap();
        }

        // Strong uptick → should trigger
        let market = MarketSnapshot {
            timestamp: chrono::Utc::now() + chrono::Duration::minutes(20),
            prices: std::collections::HashMap::from([("SOL".to_string(), 115.0)]),
            volumes: std::collections::HashMap::from([("SOL".to_string(), 2_000_000.0)]),
            funding_rates: std::collections::HashMap::new(),
            volatility: std::collections::HashMap::new(),
        };

        let signal = strat.on_tick(&market).await.unwrap();
        assert!(signal.is_some());
        assert_eq!(signal.unwrap().action, SignalAction::Buy);
    }
}   