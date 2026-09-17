use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZkError {
    #[error("Proof generation failed: {0}")]
    ProofGeneration(String),

    #[error("Proof verification failed")]
    VerificationFailed,

    #[error("Invalid witness: {0}")]
    InvalidWitness(String),

    #[error("Circuit constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Unsupported proof system: {0}")]
    UnsupportedSystem(String),
}   