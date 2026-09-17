use thiserror::Error;

#[derive(Debug, Error)]
pub enum PortfolioError {
    #[error("No positions found for user {0}")]
    NoPositions(uuid::Uuid),

    #[error("Price data unavailable for asset {0}")]
    PriceUnavailable(String),

    #[error("Valuation calculation failed: {0}")]
    ValuationFailed(String),

    #[error("Allocation target not set")]
    NoAllocationTarget,

    #[error("Drift calculation failed: {0}")]
    DriftError(String),
}   