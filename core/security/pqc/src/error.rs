use thiserror::Error;

#[derive(Debug, Error)]
pub enum PqcError {
    #[error("ML-DSA operation failed: {0}")]
    MlDsa(String),

    #[error("SLH-DSA operation failed: {0}")]
    SlhDsa(String),

    #[error("ML-KEM operation failed: {0}")]
    MlKem(String),

    #[error("Invalid key size: expected {expected}, got {actual}")]
    InvalidKeySize { expected: usize, actual: usize },

    #[error("Invalid signature size: expected {expected}, got {actual}")]
    InvalidSignatureSize { expected: usize, actual: usize },

    #[error("Verification failed")]
    VerificationFailed,

    #[error("Key generation failed: {0}")]
    KeyGenFailed(String),
}   