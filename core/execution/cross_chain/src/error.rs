use thiserror::Error;

#[derive(Debug, Error)]
pub enum CrossChainError {
    #[error("No cross-chain route found: {from_chain} → {to_chain}")]
    NoRoute { from_chain: String, to_chain: String },

    #[error("All bridges unavailable for {from} → {to}")]
    AllBridgesDown { from: String, to: String },

    #[error("Transfer amount exceeds bridge limit: {amount} > {limit}")]
    ExceedsBridgeLimit { amount: f64, limit: f64 },

    #[error("Bridge {bridge} is under maintenance")]
    BridgeUnderMaintenance(String),

    #[error("Bridge {bridge} has active exploit alert")]
    BridgeExploitAlert(String),

    #[error("Transfer stuck: no confirmation after {timeout_secs}s")]
    TransferStuck { timeout_secs: u64 },

    #[error("Token not supported on destination chain: {token}")]
    TokenNotSupported(String),

    #[error("Slippage exceeds limit: {actual_bps} > {max_bps}")]
    SlippageExceeded { actual_bps: f64, max_bps: f64 },

    #[error("Gas fee on source chain exceeds limit: {actual} > {max}")]
    GasFeeExceeded { actual: f64, max: f64 },

    #[error("Transfer failed: {0}")]
    TransferFailed(String),
}   