//! Yield Farming Optimization strategy.
//!
//! Monitors APY across lending protocols, LP positions, and staking.
//! Automatically rebalances to the highest-risk-adjusted yield.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::strategy::{Strategy, StrategyConfig, StrategySignal, StrategyState, StrategyStatus, SignalAction, MarketSnapshot};
use crate::error::StrategyError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldOptParams {
    pub min_apy: f64,
    pub max_protocol_risk_score: f64,
    pub rebalance_threshold_bps: f64,
    pub max_single_protocol_pct: f64,
    pub stablecoin_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldOpportunity {
    pub protocol: String,
    pub asset: String,
    pub apy: f64,
    pub risk_score: f64,
    pub tvl_usd: f64,
    pub risk_adjusted_apy: f64,
}

pub struct YieldOptimizationStrategy {
    id: uuid::Uuid,
    config: StrategyConfig,
    state: StrategyState,
    params: YieldOptParams,
    current_allocations: std::collections::HashMap<String, f64>,
}

impl YieldOptimizationStrategy {
    pub fn new(config: StrategyConfig, params: YieldOptParams) -> Self {
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
            current_allocations: std::collections::HashMap::new(),
        }
    }

    fn risk_adjusted_apy(&self, apy: f64, risk_score: f64) -> f64 {
        // Higher risk score = lower adjusted APY
        apy * (1.0 - risk_score / 200.0)
    }
}

#[async_trait]
impl Strategy for YieldOptimizationStrategy {
    fn id(&self) -> uuid::Uuid { self.id }
    fn name(&self) -> &str { "YieldOpt" }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn state(&self) -> &StrategyState { &self.state }

    async fn on_tick(&mut self, _market: &MarketSnapshot) -> Result<Option<StrategySignal>, StrategyError> {
        if self.state.status != StrategyStatus::Running {
            return Ok(None);
        }

        // Production: query Aave, Morpho, Yearn, Compound for current APYs
        // Compare against current allocations, rebalance if better opportunity
        // exists by more than rebalance_threshold_bps

        // Development: return None (no opportunities detected)
        Ok(None)
    }

    async fn on_fill(&mut self, signal: &StrategySignal, filled: bool, _fill_price: Option<f64>) {
        if filled {
            self.state.total_trades += 1;
            // Track new allocation
            *self.current_allocations.entry(signal.symbol.clone()).or_insert(0.0) += signal.amount_usd;
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

    #[test]
    fn test_risk_adjusted_apy() {
        let config = StrategyConfig {
            name: "Yield Opt".into(),
            symbols: vec!["USDC".into()],
            initial_capital_usd: 10_000.0,
            max_position_usd: 50_000.0,
            max_daily_spend_usd: 10_000.0,
            max_drawdown_pct: 5.0,
            stop_loss_pct: 3.0,
            take_profit_pct: None,
            params: serde_json::json!({}),
        };

        let params = YieldOptParams {
            min_apy: 3.0,
            max_protocol_risk_score: 70.0,
            rebalance_threshold_bps: 50.0,
            max_single_protocol_pct: 40.0,
            stablecoin_only: true,
        };

        let strat = YieldOptimizationStrategy::new(config, params);

        // Low risk protocol: 5% APY, risk score 20 → adjusted = 5 * (1 - 20/200) = 4.5%
        let adjusted = strat.risk_adjusted_apy(5.0, 20.0);
        assert!((adjusted - 4.5).abs() < 0.01);

        // High risk protocol: 12% APY, risk score 80 → adjusted = 12 * (1 - 80/200) = 7.2%
        let adjusted = strat.risk_adjusted_apy(12.0, 80.0);
        assert!((adjusted - 7.2).abs() < 0.01);
    }
}   