use thiserror::Error;

#[derive(Debug, Error)]
pub enum PnlError {
    #[error("Insufficient data for P&L calculation")]
    InsufficientData,

    #[error("Cost basis not available for asset {0}")]
    NoCostBasis(String),

    #[error("Calculation error: {0}")]
    Calculation(String),
}   