use thiserror::Error;

#[derive(Debug, Error)]
pub enum MpcError {
    #[error("Invalid threshold: {threshold} > {total}")]
    InvalidThreshold { threshold: u32, total: u32 },

    #[error("Share count mismatch: expected {expected}, got {actual}")]
    ShareCountMismatch { expected: u32, actual: u32 },

    #[error("HSM required for this security tier but not configured")]
    HsmRequiredButNotConfigured,

    #[error("Insufficient shares for signing: have {have}, need {need}")]
    InsufficientShares { have: u32, need: u32 },

    #[error("Key share verification failed")]
    ShareVerificationFailed,

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("HSM communication error: {0}")]
    HsmError(String),

    #[error("PQC operation failed: {0}")]
    PqcError(String),

    #[error("Invalid key material")]
    InvalidKeyMaterial,

    #[error("Key generation ceremony failed: {0}")]
    CeremonyFailed(String),

    #[error("Recovery threshold not met")]
    RecoveryThresholdNotMet,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}   