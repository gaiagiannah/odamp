//! Real-time P&L: current value vs. cost basis.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimePnl {
    pub total_value_usd: f64,
    pub total_cost_usd: f64,
    pub unrealized_gain_usd: f64,
    pub unrealized_gain_pct: f64,
    pub realized_gain_usd: f64,
    pub daily_pnl_usd: f64,
    pub daily_pnl_pct: f64,
    pub weekly_pnl_usd: f64,
    pub monthly_pnl_usd: f64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl RealtimePnl {
    pub fn calculate(
        current_value: f64,
        cost_basis: f64,
        realized_gains: f64,
        value_24h_ago: f64,
        value_7d_ago: f64,
        value_30d_ago: f64,
    ) -> Self {
        let unrealized = current_value - cost_basis;
        let unrealized_pct = if cost_basis > 0.0 { (unrealized / cost_basis) * 100.0 } else { 0.0 };

        Self {
            total_value_usd: current_value,
            total_cost_usd: cost_basis,
            unrealized_gain_usd: unrealized,
            unrealized_gain_pct: unrealized_pct,
            realized_gain_usd: realized_gains,
            daily_pnl_usd: current_value - value_24h_ago,
            daily_pnl_pct: if value_24h_ago > 0.0 { ((current_value - value_24h_ago) / value_24h_ago) * 100.0 } else { 0.0 },
            weekly_pnl_usd: current_value - value_7d_ago,
            monthly_pnl_usd: current_value - value_30d_ago,
            updated_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pnl_calculation() {
        let pnl = RealtimePnl::calculate(
            110_000.0, // current
            100_000.0, // cost
            5_000.0,   // realized
            108_000.0, // 24h ago
            105_000.0, // 7d ago
            95_000.0,  // 30d ago
        );

        assert_eq!(pnl.unrealized_gain_usd, 10_000.0);
        assert_eq!(pnl.unrealized_gain_pct, 10.0);
        assert_eq!(pnl.daily_pnl_usd, 2_000.0);
        assert_eq!(pnl.monthly_pnl_usd, 15_000.0);
    }
}   