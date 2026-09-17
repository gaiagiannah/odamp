//! Mean Reversion strategy.
//!
//! Buys when price deviates below the moving average by more than
//! N standard deviations; sells when it reverts to the mean.
//! Works best in ranging markets.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::strategy::{Strategy, StrategyConfig, StrategySignal, StrategyState, StrategyStatus, SignalAction, MarketSnapshot};
use crate::error::StrategyError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeanReversionParams {
    pub lookback_periods: u32,
    pub entry_std_devs: f64,
    pub exit_std_devs: f64,
    pub order_size_usd: f64,
}

pub struct MeanReversionStrategy {
    id: uuid::Uuid,
    config: StrategyConfig,
    state: StrategyState,
    params: MeanReversionParams,
    price_history: VecDeque<f64>,
    in_position: bool,
    entry_price: Option<f64>,
}

impl MeanReversionStrategy {
    pub fn new(config: StrategyConfig, params: MeanReversionParams) -> Self {
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
            in_position: false,
            entry_price: None,
        }
    }

    fn moving_average(&self) -> f64 {
        if self.price_history.is_empty() { return 0.0; }
        self.price_history.iter().sum::<f64>() / self.price_history.len() as f64
    }

    fn std_dev(&self) -> f64 {
        let n = self.price_history.len();
        if n < 2 { return 0.0; }
        let mean = self.moving_average();
        let variance = self.price_history.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / n as f64;
        variance.sqrt()
    }

    fn z_score(&self, price: f64) -> f64 {
        let std = self.std_dev();
        if std == 0.0 { return 0.0; }
        (price - self.moving_average()) / std
    }
}

#[async_trait]
impl Strategy for MeanReversionStrategy {
    fn id(&self) -> uuid::Uuid { self.id }
    fn name(&self) -> &str { "MeanReversion" }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn state(&self) -> &StrategyState { &self.state }

    async fn on_tick(&mut self, market: &MarketSnapshot) -> Result<Option<StrategySignal>, StrategyError> {
        if self.state.status != StrategyStatus::Running {
            return Ok(None);
        }

        let symbol = &self.config.symbols[0];
        let price = market.prices.get(symbol).copied().unwrap_or(0.0);
        if price == 0.0 {
            return Err(StrategyError::NoMarketData(self.id));
        }

        self.price_history.push_back(price);
        if self.price_history.len() > self.params.lookback_periods as usize {
            self.price_history.pop_front();
        }

        // Need enough data
        if self.price_history.len() < (self.params.lookback_periods / 2) as usize {
            return Ok(None);
        }

        let z = self.z_score(price);

        // Entry: price is N std devs below mean
        if !self.in_position && z < -self.params.entry_std_devs {
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
                reason: format!("MeanReversion: entry at z={:.2} (threshold -{:.2})", z, self.params.entry_std_devs),
                confidence: (z.abs() / (self.params.entry_std_devs * 2.0)).min(1.0),
                timestamp: market.timestamp,
            }));
        }

        // Exit: price reverts to within exit threshold of mean
        if self.in_position && z > -self.params.exit_std_devs {
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
                reason: format!("MeanReversion: exit at z={:.2} (P&L ${:.2})", z, pnl),
                confidence: 0.85,
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
            return Some(format!("Max drawdown exceeded"));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mean_reversion_entry() {
        let config = StrategyConfig {
            name: "ETH MR".into(),
            symbols: vec!["ETH".into()],
            initial_capital_usd: 5_000.0,
            max_position_usd: 10_000.0,
            max_daily_spend_usd: 5_000.0,
            max_drawdown_pct: 15.0,
            stop_loss_pct: 10.0,
            take_profit_pct: None,
            params: serde_json::json!({}),
        };

        let params = MeanReversionParams {
            lookback_periods: 50,
            entry_std_devs: 2.0,
            exit_std_devs: 0.5,
            order_size_usd: 500.0,
        };

        let mut strat = MeanReversionStrategy::new(config, params);

        // Feed stable prices around 3000
        for i in 0..40 {
            let market = MarketSnapshot {
                timestamp: chrono::Utc::now() + chrono::Duration::minutes(i),
                prices: std::collections::HashMap::from([("ETH".to_string(), 3000.0 + (i as f64 % 5) * 2.0)]),
                volumes: std::collections::HashMap::new(),
                funding_rates: std::collections::HashMap::new(),
                volatility: std::collections::HashMap::new(),
            };
            let _ = strat.on_tick(&market).await.unwrap();
        }

        // Sharp drop → should trigger entry
        let market = MarketSnapshot {
            timestamp: chrono::Utc::now() + chrono::Duration::minutes(41),
            prices: std::collections::HashMap::from([("ETH".to_string(), 2800.0)]),
            volumes: std::collections::HashMap::new(),
            funding_rates: std::collections::HashMap::new(),
            volatility: std::collections::HashMap::new(),
        };

        let signal = strat.on_tick(&market).await.unwrap();
        assert!(signal.is_some());
        assert_eq!(signal.unwrap().action, SignalAction::Buy);
    }
}   