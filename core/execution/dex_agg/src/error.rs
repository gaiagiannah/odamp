use thiserror::Error;

#[derive(Debug, Error)]
pub enum DexAggError {
    #[error("No DEX route found: {from} → {to} on {chain}")]
    NoRoute { from: String, to: String, chain: String },

    #[error("All DEX APIs failed: {0}")]
    AllApisFailed(String),

    #[error("Route slippage exceeds limit: {actual}% > {max}%")]
    SlippageExceeded { actual: f64, max: f64 },

    #[error("Insufficient liquidity in route: need {needed}, available {available}")]
    InsufficientLiquidity { needed: f64, available: f64 },

    #[error("Token not found on chain {chain}: {token}")]
    TokenNotFound { chain: String, token: String },

    #[error("API error from {source}: {status}")]
    ApiError { source: String, status: u16 },

    #[error("Route execution failed: {0}")]
    ExecutionFailed(String),
}   