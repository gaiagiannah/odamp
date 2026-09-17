//! Arbitrage strategy: cross-venue, cross-chain, and triangular.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::strategy::{Strategy, StrategyConfig, StrategySignal, StrategyState, StrategyStatus, SignalAction, MarketSnapshot};
use crate::error::StrategyError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArbType {
    CrossVenue,
    CrossChain,
    Triangular,
    FundingRate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageParams {
    pub arb_type: ArbType,
    pub min_profit_bps: f64,
    pub max_position_usd: f64,
    pub execution_timeout_secs: u32,
}

pub struct ArbitrageStrategy {
    id: uuid::Uuid,
    config: StrategyConfig,
    state: StrategyState,
    params: ArbitrageParams,
}

impl ArbitrageStrategy {
    pub fn new(config: StrategyConfig, params: ArbitrageParams) -> Self {
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
        }
    }

    /// Detects cross-venue arbitrage opportunity.
    fn detect_cross_venue(&self, market: &MarketSnapshot) -> Option<StrategySignal> {
        // In production: compare prices across venues in real-time
        // For development: check if funding rate implies arb
        let symbol = &self.config.symbols[0];
        let funding = market.funding_rates.get(symbol).copied().unwrap_or(0.0);

        // Annualized funding > threshold → long spot, short perp
        let annualized = funding * 3.0 * 365.0 * 100.0; // funding is per 8h
        if annualized > self.params.min_profit_bps * 100.0 {
            return Some(StrategySignal {
                strategy_id: self.id,
                symbol: symbol.clone(),
                action: SignalAction::Buy,
                amount: self.params.max_position_usd / market.prices.get(symbol).copied().unwrap_or(1.0),
                amount_usd: self.params.max_position_usd,
                limit_price: None,
                reason: format!("FundingRateArb: annualized {:.1}% > threshold {:.1}%", annualized, self.params.min_profit_bps * 100.0),
                confidence: 0.9,
                timestamp: market.timestamp,
            });
        }

        None
    }
}

#[async_trait]
impl Strategy for ArbitrageStrategy {
    fn id(&self) -> uuid::Uuid { self.id }
    fn name(&self) -> &str { "Arbitrage" }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn state(&self) -> &StrategyState { &self.state }

    async fn on_tick(&mut self, market: &MarketSnapshot) -> Result<Option<StrategySignal>, StrategyError> {
        if self.state.status != StrategyStatus::Running {
            return Ok(None);
        }

        let signal = match self.params.arb_type {
            ArbType::FundingRate => self.detect_cross_venue(market),
            ArbType::CrossVenue | ArbType::CrossChain | ArbType::Triangular => {
                // Production: query multiple venues/chains for price discrepancy
                None
            }
        };

        if let Some(sig) = &signal {
            self.state.total_trades += 1;
        }

        Ok(signal)
    }

    async fn on_fill(&mut self, signal: &StrategySignal, filled: bool, _fill_price: Option<f64>) {
        if filled {
            // Arbitrage P&L is the spread captured
            self.state.total_pnl_usd += signal.amount_usd * (self.params.min_profit_bps / 10_000.0);
        }
    }

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
    async fn test_funding_rate_arb() {
        let config = StrategyConfig {
            name: "BTC Funding Arb".into(),
            symbols: vec!["BTC".into()],
            initial_capital_usd: 50_000.0,
            max_position_usd: 25_000.0,
            max_daily_spend_usd: 50_000.0,
            max_drawdown_pct: 10.0,
            stop_loss_pct: 5.0,
            take_profit_pct: None,
            params: serde_json::json!({}),
        };

        let params = ArbitrageParams {
            arb_type: ArbType::FundingRate,
            min_profit_bps: 50.0,
            max_position_usd: 10_000.0,
            execution_timeout_secs: 30,
        };

        let mut strat = ArbitrageStrategy::new(config, params);

        // High funding rate → should trigger
        let market = MarketSnapshot {
            timestamp: chrono::Utc::now(),
            prices: std::collections::HashMap::from([("BTC".to_string(), 100_000.0)]),
            volumes: std::collections::HashMap::new(),
            funding_rates: std::collections::HashMap::from([("BTC".to_string(), 0.0005)]), // 0.05% per 8h = 54.75% annualized
            volatility: std::collections::HashMap::new(),
        };

        let signal = strat.on_tick(&market).await.unwrap();
        assert!(signal.is_some());
        assert!(signal.unwrap().reason.contains("FundingRateArb"));
    }

    #[tokio::test]
    async fn test_no_arb_low_funding() {
        let config = StrategyConfig {
            name: "BTC Funding Arb".into(),
            symbols: vec!["BTC".into()],
            initial_capital_usd: 50_000.0,
            max_position_usd: 25_000.0,
            max_daily_spend_usd: 50_000.0,
            max_drawdown_pct: 10.0,
            stop_loss_pct: 5.0,
            take_profit_pct: None,
            params: serde_json::json!({}),
        };

        let params = ArbitrageParams {
            arb_type: ArbType::FundingRate,
            min_profit_bps: 500.0, // very high threshold
            max_position_usd: 10_000.0,
            execution_timeout_secs: 30,
        };

        let mut strat = ArbitrageStrategy::new(config, params);

        let market = MarketSnapshot {
            timestamp: chrono::Utc::now(),
            prices: std::collections::HashMap::from([("BTC".to_string(), 100_000.0)]),
            volumes: std::collections::HashMap::new(),
            funding_rates: std::collections::HashMap::from([("BTC".to_string(), 0.0001)]),
            volatility: std::collections::HashMap::new(),
        };

        let signal = strat.on_tick(&market).await.unwrap();
        assert!(signal.is_none());
    }
}   