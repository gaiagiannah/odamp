//! Grid Trading strategy.
//!
//! Places buy orders below and sell orders above the current price
//! at fixed intervals. Profits from oscillation in ranging markets.
//! Loses in trending markets (mitigated by stop-loss).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::strategy::{Strategy, StrategyConfig, StrategySignal, StrategyState, StrategyStatus, SignalAction, MarketSnapshot};
use crate::error::StrategyError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridParams {
    pub grid_levels: u32,
    pub grid_spacing_pct: f64,
    pub order_size_usd: f64,
    pub upper_bound_pct: f64,
    pub lower_bound_pct: f64,
}

pub struct GridStrategy {
    id: uuid::Uuid,
    config: StrategyConfig,
    state: StrategyState,
    params: GridParams,
    center_price: Option<f64>,
    active_buys: Vec<f64>,
    active_sells: Vec<f64>,
}

impl GridStrategy {
    pub fn new(config: StrategyConfig, params: GridParams, center_price: f64) -> Self {
        let id = uuid::Uuid::new_v4();
        let mut active_buys = vec![];
        let mut active_sells = vec![];

        for i in 1..=params.grid_levels {
            let buy_price = center_price * (1.0 - (i as f64 * params.grid_spacing_pct / 100.0));
            let sell_price = center_price * (1.0 + (i as f64 * params.grid_spacing_pct / 100.0));
            if buy_price > 0.0 {
                active_buys.push(buy_price);
            }
            active_sells.push(sell_price);
        }

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
            center_price: Some(center_price),
            active_buys,
            active_sells,
        }
    }

    fn check_grid_hit(&self, price: f64) -> Option<StrategySignal> {
        let symbol = &self.config.symbols[0];

        // Check if price hit a buy level
        if let Some(idx) = self.active_buys.iter().position(|&level| price <= level) {
            let level = self.active_buys.remove(idx);
            return Some(StrategySignal {
                strategy_id: self.id,
                symbol: symbol.clone(),
                action: SignalAction::Buy,
                amount: self.params.order_size_usd / level,
                amount_usd: self.params.order_size_usd,
                limit_price: Some(level),
                reason: format!("Grid: buy level hit at ${:.2}", level),
                confidence: 0.8,
                timestamp: chrono::Utc::now(),
            });
        }

        // Check if price hit a sell level
        if let Some(idx) = self.active_sells.iter().position(|&level| price >= level) {
            let level = self.active_sells.remove(idx);
            return Some(StrategySignal {
                strategy_id: self.id,
                symbol: symbol.clone(),
                action: SignalAction::Sell,
                amount: self.params.order_size_usd / level,
                amount_usd: self.params.order_size_usd,
                limit_price: Some(level),
                reason: format!("Grid: sell level hit at ${:.2}", level),
                confidence: 0.8,
                timestamp: chrono::Utc::now(),
            });
        }

        None
    }
}

#[async_trait]
impl Strategy for GridStrategy {
    fn id(&self) -> uuid::Uuid { self.id }
    fn name(&self) -> &str { "Grid" }
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

        // Check bounds
        if let Some(center) = self.center_price {
            let upper = center * (1.0 + self.params.upper_bound_pct / 100.0);
            let lower = center * (1.0 - self.params.lower_bound_pct / 100.0);

            if price > upper || price < lower {
                self.stop(&format!("Price ${:.2} outside grid bounds [${:.2}, ${:.2}]", price, lower, upper));
                return Ok(None);
            }
        }

        // Check risk limits
        if let Some(reason) = self.check_risk_limits() {
            self.stop(&reason);
            return Ok(None);
        }

        // Check grid levels
        if let Some(signal) = self.check_grid_hit(price) {
            self.state.total_trades += 1;
            Ok(Some(signal))
        } else {
            Ok(None)
        }
    }

    async fn on_fill(&mut self, signal: &StrategySignal, filled: bool, fill_price: Option<f64>) {
        if filled {
            self.state.total_pnl_usd += if signal.action == SignalAction::Sell {
                self.params.grid_spacing_pct / 2.0 * self.params.order_size_usd / 100.0
            } else {
                0.0
            };
        }
    }

    fn stop(&mut self, reason: &str) {
        self.state.status = StrategyStatus::Stopped;
        self.state.stopped_at = Some(chrono::Utc::now());
        tracing::info!(strategy = %self.id, reason, "Grid strategy stopped");
    }

    fn check_risk_limits(&self) -> Option<String> {
        if self.state.current_drawdown_pct > self.config.max_drawdown_pct {
            return Some(format!("Max drawdown {:.1}% exceeded", self.config.max_drawdown_pct));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_grid_buy_signal() {
        let config = StrategyConfig {
            name: "BTC Grid".into(),
            symbols: vec!["BTC".into()],
            initial_capital_usd: 10_000.0,
            max_position_usd: 50_000.0,
            max_daily_spend_usd: 10_000.0,
            max_drawdown_pct: 15.0,
            stop_loss_pct: 10.0,
            take_profit_pct: None,
            params: serde_json::json!({}),
        };

        let params = GridParams {
            grid_levels: 5,
            grid_spacing_pct: 2.0,
            order_size_usd: 500.0,
            upper_bound_pct: 15.0,
            lower_bound_pct: 15.0,
        };

        let center = 100_000.0;
        let mut grid = GridStrategy::new(config, params, center);

        // Price drops 2% → should hit first buy level (98,000)
        let market = MarketSnapshot {
            timestamp: chrono::Utc::now(),
            prices: std::collections::HashMap::from([("BTC".to_string(), 97_900.0)]),
            volumes: std::collections::HashMap::new(),
            funding_rates: std::collections::HashMap::new(),
            volatility: std::collections::HashMap::new(),
        };

        let signal = grid.on_tick(&market).await.unwrap();
        assert!(signal.is_some());
        assert_eq!(signal.unwrap().action, SignalAction::Buy);
    }

    #[tokio::test]
    async fn test_grid_stops_outside_bounds() {
        let config = StrategyConfig {
            name: "BTC Grid".into(),
            symbols: vec!["BTC".into()],
            initial_capital_usd: 10_000.0,
            max_position_usd: 50_000.0,
            max_daily_spend_usd: 10_000.0,
            max_drawdown_pct: 15.0,
            stop_loss_pct: 10.0,
            take_profit_pct: None,
            params: serde_json::json!({}),
        };

        let params = GridParams {
            grid_levels: 5,
            grid_spacing_pct: 2.0,
            order_size_usd: 500.0,
            upper_bound_pct: 10.0,
            lower_bound_pct: 10.0,
        };

        let mut grid = GridStrategy::new(config, params, 100_000.0);

        // Price above upper bound (110,000)
        let market = MarketSnapshot {
            timestamp: chrono::Utc::now(),
            prices: std::collections::HashMap::from([("BTC".to_string(), 112_000.0)]),
            volumes: std::collections::HashMap::new(),
            funding_rates: std::collections::HashMap::new(),
            volatility: std::collections::HashMap::new(),
        };

        let _ = grid.on_tick(&market).await.unwrap();
        assert_eq!(grid.state().status, StrategyStatus::Stopped);
    }
}   