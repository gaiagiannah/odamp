//! Dollar Cost Averaging strategy.
//!
//! Buys a fixed amount at regular intervals regardless of price.
//! Reduces the impact of volatility by smoothing entry price.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::strategy::{Strategy, StrategyConfig, StrategySignal, StrategyState, StrategyStatus, MarketSnapshot};
use crate::error::StrategyError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcaParams {
    pub amount_usd: f64,
    pub interval_hours: u32,
    pub max_purchases: Option<u32>,
    pub price_cap: Option<f64>, // don't buy above this price
}

pub struct DcaStrategy {
    id: uuid::Uuid,
    config: StrategyConfig,
    state: StrategyState,
    params: DcaParams,
    last_purchase: Option<chrono::DateTime<chrono::Utc>>,
    purchase_count: u32,
}

impl DcaStrategy {
    pub fn new(config: StrategyConfig, params: DcaParams) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            state: StrategyState {
                id: uuid::Uuid::new_v4(),
                status: StrategyStatus::Running,
                started_at: chrono::Utc::now(),
                stopped_at: None,
                total_trades: 0,
                total_pnl_usd: 0.0,
                current_drawdown_pct: 0.0,
                daily_spent_usd: 0.0,
                positions: vec![],
            },
            config,
            params,
            last_purchase: None,
            purchase_count: 0,
        }
    }
}

#[async_trait]
impl Strategy for DcaStrategy {
    fn id(&self) -> uuid::Uuid { self.id }
    fn name(&self) -> &str { "DCA" }
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

        // Check price cap
        if let Some(cap) = self.params.price_cap {
            if price > cap {
                return Ok(None); // skip this purchase
            }
        }

        // Check interval
        let should_buy = match self.last_purchase {
            None => true,
            Some(last) => {
                (market.timestamp - last).num_hours() >= self.params.interval_hours as i64
            }
        };

        if !should_buy {
            return Ok(None);
        }

        // Check max purchases
        if let Some(max) = self.params.max_purchases {
            if self.purchase_count >= max {
                self.stop("Max purchases reached");
                return Ok(None);
            }
        }

        // Check daily limit
        if self.state.daily_spent_usd + self.params.amount_usd > self.config.max_daily_spend_usd {
            return Err(StrategyError::DailyLimitReached {
                spent: self.state.daily_spent_usd,
                limit: self.config.max_daily_spend_usd,
            });
        }

        let amount = self.params.amount_usd / price;

        self.last_purchase = Some(market.timestamp);
        self.purchase_count += 1;
        self.state.total_trades += 1;
        self.state.daily_spent_usd += self.params.amount_usd;

        Ok(Some(StrategySignal {
            strategy_id: self.id,
            symbol: symbol.clone(),
            action: crate::strategy::SignalAction::Buy,
            amount,
            amount_usd: self.params.amount_usd,
            limit_price: None,
            reason: format!("DCA: purchase #{}/{} at ${:.2}", self.purchase_count, self.params.max_purchases.unwrap_or(u32::MAX), price),
            confidence: 0.95,
            timestamp: market.timestamp,
        }))
    }

    async fn on_fill(&mut self, signal: &StrategySignal, filled: bool, fill_price: Option<f64>) {
        if filled {
            if let Some(price) = fill_price {
                let existing = self.state.positions.iter_mut()
                    .find(|p| p.symbol == signal.symbol);
                if let Some(pos) = existing {
                    let new_amount = pos.amount + signal.amount;
                    pos.avg_cost = (pos.avg_cost * pos.amount + price * signal.amount) / new_amount;
                    pos.amount = new_amount;
                    pos.current_value_usd = new_amount * price;
                    pos.unrealized_pnl_usd = (price - pos.avg_cost) * new_amount;
                } else {
                    self.state.positions.push(crate::strategy::StrategyPosition {
                        symbol: signal.symbol.clone(),
                        amount: signal.amount,
                        avg_cost: price,
                        current_value_usd: signal.amount * price,
                        unrealized_pnl_usd: 0.0,
                    });
                }
            }
        }
    }

    fn stop(&mut self, reason: &str) {
        self.state.status = StrategyStatus::Stopped;
        self.state.stopped_at = Some(chrono::Utc::now());
        tracing::info!(strategy = %self.id, reason, "DCA strategy stopped");
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
    async fn test_dca_first_purchase() {
        let config = StrategyConfig {
            name: "BTC DCA".into(),
            symbols: vec!["BTC".into()],
            initial_capital_usd: 10_000.0,
            max_position_usd: 50_000.0,
            max_daily_spend_usd: 5_000.0,
            max_drawdown_pct: 20.0,
            stop_loss_pct: 10.0,
            take_profit_pct: None,
            params: serde_json::json!({}),
        };

        let params = DcaParams {
            amount_usd: 500.0,
            interval_hours: 24,
            max_purchases: Some(20),
            price_cap: None,
        };

        let mut dca = DcaStrategy::new(config, params);

        let market = MarketSnapshot {
            timestamp: chrono::Utc::now(),
            prices: std::collections::HashMap::from([("BTC".to_string(), 100_000.0)]),
            volumes: std::collections::HashMap::new(),
            funding_rates: std::collections::HashMap::new(),
            volatility: std::collections::HashMap::new(),
        };

        let signal = dca.on_tick(&market).await.unwrap();
        assert!(signal.is_some());
        let sig = signal.unwrap();
        assert_eq!(sig.action, crate::strategy::SignalAction::Buy);
        assert!((sig.amount_usd - 500.0).abs() < 0.01);
        assert!((sig.amount - 0.005).abs() < 0.0001); // 500 / 100000
    }

    #[tokio::test]
    async fn test_dca_respects_interval() {
        let config = StrategyConfig {
            name: "BTC DCA".into(),
            symbols: vec!["BTC".into()],
            initial_capital_usd: 10_000.0,
            max_position_usd: 50_000.0,
            max_daily_spend_usd: 5_000.0,
            max_drawdown_pct: 20.0,
            stop_loss_pct: 10.0,
            take_profit_pct: None,
            params: serde_json::json!({}),
        };

        let params = DcaParams {
            amount_usd: 500.0,
            interval_hours: 24,
            max_purchases: None,
            price_cap: None,
        };

        let mut dca = DcaStrategy::new(config, params);

        let market1 = MarketSnapshot {
            timestamp: chrono::Utc::now(),
            prices: std::collections::HashMap::from([("BTC".to_string(), 100_000.0)]),
            volumes: std::collections::HashMap::new(),
            funding_rates: std::collections::HashMap::new(),
            volatility: std::collections::HashMap::new(),
        };

        // First purchase
        let sig1 = dca.on_tick(&market1).await.unwrap();
        assert!(sig1.is_some());

        // Second tick 1 hour later — should NOT buy (interval is 24h)
        let market2 = MarketSnapshot {
            timestamp: chrono::Utc::now() + chrono::Duration::hours(1),
            prices: std::collections::HashMap::from([("BTC".to_string(), 101_000.0)]),
            volumes: std::collections::HashMap::new(),
            funding_rates: std::collections::HashMap::new(),
            volatility: std::collections::HashMap::new(),
        };

        let sig2 = dca.on_tick(&market2).await.unwrap();
        assert!(sig2.is_none());

        // Third tick 25 hours after first — SHOULD buy
        let market3 = MarketSnapshot {
            timestamp: chrono::Utc::now() + chrono::Duration::hours(25),
            prices: std::collections::HashMap::from([("BTC".to_string(), 99_000.0)]),
            volumes: std::collections::HashMap::new(),
            funding_rates: std::collections::HashMap::new(),
            volatility: std::collections::HashMap::new(),
        };

        let sig3 = dca.on_tick(&market3).await.unwrap();
        assert!(sig3.is_some());
    }

    #[tokio::test]
    async fn test_dca_price_cap() {
        let config = StrategyConfig {
            name: "BTC DCA Capped".into(),
            symbols: vec!["BTC".into()],
            initial_capital_usd: 10_000.0,
            max_position_usd: 50_000.0,
            max_daily_spend_usd: 5_000.0,
            max_drawdown_pct: 20.0,
            stop_loss_pct: 10.0,
            take_profit_pct: None,
            params: serde_json::json!({}),
        };

        let params = DcaParams {
            amount_usd: 500.0,
            interval_hours: 1,
            max_purchases: None,
            price_cap: Some(100_000.0), // don't buy above $100K
        };

        let mut dca = DcaStrategy::new(config, params);

        // Price above cap — should skip
        let market = MarketSnapshot {
            timestamp: chrono::Utc::now(),
            prices: std::collections::HashMap::from([("BTC".to_string(), 105_000.0)]),
            volumes: std::collections::HashMap::new(),
            funding_rates: std::collections::HashMap::new(),
            volatility: std::collections::HashMap::new(),
        };

        let signal = dca.on_tick(&market).await.unwrap();
        assert!(signal.is_none());
    }
}     