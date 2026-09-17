//! Complete portfolio snapshot: the single source of truth for
//! what a user owns, what it's worth, and how it's allocated.

use serde::{Deserialize, Serialize};
use odamp_indexer::positions::{AssetType, Position};
use odamp_indexer::chain::ChainId;

/// A complete, point-in-time portfolio snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub user_id: uuid::Uuid,
    pub generated_at: chrono::DateTime<chrono::Utc>,

    // Totals
    pub total_value_usd: f64,
    pub total_cost_basis_usd: f64,
    pub total_unrealized_gain_usd: f64,
    pub total_unrealized_gain_pct: f64,

    // By asset type
    pub by_asset_type: Vec<AssetTypeSummary>,

    // By chain
    pub by_chain: Vec<ChainSummary>,

    // By protocol (DeFi)
    pub by_protocol: Vec<ProtocolSummary>,

    // Individual positions (top N by value)
    pub top_positions: Vec<PositionValue>,

    // Risk metrics
    pub risk: RiskSummary,

    // Tax
    pub tax_exposure_usd: f64,

    // Concentration warnings
    pub concentration_alerts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTypeSummary {
    pub asset_type: AssetType,
    pub value_usd: f64,
    pub pct_of_total: f64,
    pub position_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSummary {
    pub chain: ChainId,
    pub value_usd: f64,
    pub pct_of_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSummary {
    pub protocol: String,
    pub category: String,
    pub value_usd: f64,
    pub apy: Option<f64>,
    pub health_factor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionValue {
    pub symbol: String,
    pub chain: ChainId,
    pub amount: f64,
    pub value_usd: f64,
    pub pct_of_total: f64,
    pub change_24h_pct: f64,
    pub change_7d_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSummary {
    pub sharpe_ratio: f64,
    pub max_drawdown_30d: f64,
    pub var_95_1d: f64,
    pub cvar_95_1d: f64,
    pub realized_vol_30d: f64,
    pub defi_exposure_pct: f64,
    pub stablecoin_pct: f64,
    pub concentration_max_pct: f64,
}

impl PortfolioSnapshot {
    pub fn is_healthy(&self) -> bool {
        // No critical risk flags
        self.risk.var_95_1d < 5.0 && self.risk.defi_exposure_pct < 50.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_construction() {
        let snapshot = PortfolioSnapshot {
            user_id: uuid::Uuid::new_v4(),
            generated_at: chrono::Utc::now(),
            total_value_usd: 100_000.0,
            total_cost_basis_usd: 80_000.0,
            total_unrealized_gain_usd: 20_000.0,
            total_unrealized_gain_pct: 25.0,
            by_asset_type: vec![],
            by_chain: vec![],
            by_protocol: vec![],
            top_positions: vec![],
            risk: RiskSummary {
                sharpe_ratio: 1.5,
                max_drawdown_30d: 5.2,
                var_95_1d: 2.1,
                cvar_95_1d: 3.4,
                realized_vol_30d: 35.0,
                defi_exposure_pct: 30.0,
                stablecoin_pct: 20.0,
                concentration_max_pct: 40.0,
            },
            tax_exposure_usd: 5_000.0,
            concentration_alerts: vec!["BTC is 40% of portfolio".into()],
        };

        assert!(snapshot.is_healthy());
        assert_eq!(snapshot.total_unrealized_gain_usd, 20_000.0);
    }
}   