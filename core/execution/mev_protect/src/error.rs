use thiserror::Error;

#[derive(Debug, Error)]
pub enum MevError {
    #[error("Transaction simulation failed: {0}")]
    SimulationFailed(String),

    #[error("MEV attack detected: {0}")]
    AttackDetected(String),

    #[error("Slippage exceeds hard limit: {actual_bps} > {max_bps}")]
    SlippageHardLimit { actual_bps: f64, max_bps: f64 },

    #[error("Private mempool unavailable: {0}")]
    MempoolUnavailable(String),

    #[error("Bundle rejected by sequencer: {0}")]
    BundleRejected(String),

    #[error("Transaction would result in negative P&L after MEV: loss ${0}")]
    NegativePnl(f64),
}   