use thiserror::Error;

#[derive(Debug, Error)]
pub enum AllocationError {
    #[error("Allocation targets must sum to 100%: got {0}%")]
    TargetsNot100(f64),

    #[error("Unknown asset class: {0}")]
    UnknownAssetClass(String),

    #[error("Rebalance would exceed spending limit: {amount} > {limit}")]
    ExceedsLimit { amount: f64, limit: f64 },

    #[error("Insufficient liquidity to rebalance {asset}: need {needed}, available {available}")]
    InsufficientLiquidity { asset: String, needed: f64, available: f64 },

    #[error("Rebalance blocked by risk overlay: {0}")]
    RiskBlocked(String),
}   