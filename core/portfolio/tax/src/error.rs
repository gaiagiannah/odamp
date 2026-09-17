use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaxError {
    #[error("No cost basis found for asset {0}")]
    NoCostBasis(String),

    #[error("Unsupported jurisdiction: {0}")]
    UnsupportedJurisdiction(String),

    #[error("Tax year not specified")]
    NoTaxYear,

    #[error("Report generation failed: {0}")]
    ReportGeneration(String),

    #[error("Insufficient transaction history for calculation")]
    InsufficientHistory,

    #[error("Conflicting cost basis methods for same asset")]
    ConflictingMethods,
}   