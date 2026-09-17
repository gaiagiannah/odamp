//! Slippage estimation and protection for DEX swaps.

use serde::{Deserialize, Serialize};

/// Slippage configuration for a swap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageConfig {
    pub max_slippage_bps: f64,
    pub min_slippage_bps: f64,
    pub price_impact_threshold_bps: f64,
    pub use_oracle_price: bool,
}

impl Default for SlippageConfig {
    fn default() -> Self {
        Self {
            max_slippage_bps: 100.0,   // 1%
            min_slippage_bps: 5.0,     // 0.05%
            price_impact_threshold_bps: 500.0, // 5%
            use_oracle_price: true,
        }
    }
}

/// Estimates slippage for a given trade size against pool depth.
pub fn estimate_slippage(
    trade_size_usd: f64,
    pool_liquidity_usd: f64,
    pool_fee_bps: f64,
) -> f64 {
    if pool_liquidity_usd == 0.0 {
        return f64::MAX;
    }

    // Constant product formula: slippage ≈ trade_size / (2 * liquidity)
    let price_impact = (trade_size_usd / (2.0 * pool_liquidity_usd)) * 10_000.0;
    let total = price_impact + pool_fee_bps;
    total
}

/// Validates that a route's slippage is within acceptable bounds.
pub fn validate_slippage(
    route_slippage_bps: f64,
    config: &SlippageConfig,
) -> Result<(), String> {
    if route_slippage_bps > config.max_slippage_bps {
        return Err(format!(
            "Slippage {:.2} bps exceeds max {:.2} bps",
            route_slippage_bps, config.max_slippage_bps
        ));
    }
    if route_slippage_bps < config.min_slippage_bps {
        // Suspiciously low — might be a stale quote
        return Err(format!(
            "Slippage {:.2} bps below minimum {:.2} bps (possibly stale quote)",
            route_slippage_bps, config.min_slippage_bps
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slippage_estimation() {
        // $10K trade against $1M pool with 30 bps fee
        let slip = estimate_slippage(10_000.0, 1_000_000.0, 30.0);
        // Price impact: 10000 / (2 * 1000000) * 10000 = 50 bps
        // Total: 50 + 30 = 80 bps
        assert!((slip - 80.0).abs() < 0.1);
    }

    #[test]
    fn test_slippage_validation() {
        let config = SlippageConfig::default();
        assert!(validate_slippage(50.0, &config).is_ok());
        assert!(validate_slippage(200.0, &config).is_err()); // too high
        assert!(validate_slippage(1.0, &config).is_err()); // too low
    }
}   