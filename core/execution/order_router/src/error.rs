use thiserror::Error;

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("No available venues for {symbol}")]
    NoVenues(String),

    #[error("All venues returned errors: {0}")]
    AllVenuesFailed(String),

    #[error("Order size exceeds venue limit: {size} > {limit}")]
    ExceedsVenueLimit { size: f64, limit: f64 },

    #[error("Insufficient liquidity across all venues: need {needed}, available {available}")]
    InsufficientLiquidity { needed: f64, available: f64 },

    #[error("Routing timeout after {0}ms")]
    Timeout(u64),

    #[error("Venue {venue} unavailable: {reason}")]
    VenueUnavailable { venue: String, reason: String },

    #[error("Cross-chain route not found: {from} → {to}")]
    NoCrossChainRoute { from: String, to: String },

    #[error("MEV protection failed: {0}")]
    MevProtection(String),

    #[error("Slippage exceeds max: {actual_bps} > {max_bps}")]
    SlippageExceeded { actual_bps: f64, max_bps: f64 },
}   