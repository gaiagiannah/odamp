//! DEX route representation: multi-hop paths through intermediate tokens.

use serde::{Deserialize, Serialize};

/// A complete DEX route (may involve multiple hops).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexRoute {
    pub source_dex: String,       // "1inch", "jupiter", "uniswap", "curve"
    pub chain: String,
    pub from_token: String,
    pub to_token: String,
    pub from_amount: f64,
    pub to_amount_min: f64,       // after slippage
    pub to_amount_expected: f64,
    pub steps: Vec<RouteStep>,
    pub total_fee_bps: f64,
    pub estimated_gas_usd: f64,
    pub execution_time_ms: u64,
    pub is_optimal: bool,
}

/// A single step in a multi-hop route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStep {
    pub dex: String,
    pub pool: String,
    pub from_token: String,
    pub to_token: String,
    pub from_amount: f64,
    pub to_amount: f64,
    pub pool_fee_bps: f64,
    pub liquidity_depth: f64,
}

impl DexRoute {
    pub fn slippage_pct(&self) -> f64 {
        if self.to_amount_expected > 0.0 {
            ((self.to_amount_expected - self.to_amount_min) / self.to_amount_expected) * 100.0
        } else { 0.0 }
    }

    pub fn total_cost_bps(&self) -> f64 {
        self.total_fee_bps + self.slippage_pct() * 100.0
    }
}   