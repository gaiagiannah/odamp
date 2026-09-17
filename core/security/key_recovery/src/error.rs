use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error("Insufficient recovery shares: have {have}, need {need}")]
    InsufficientShares { have: u32, need: u32 },

    #[error("Duplicate recovery share: {0}")]
    DuplicateShare(u32),

    #[error("Recovery request not yet eligible (time lock active)")]
    TimeLockActive,

    #[error("Trusted contact not found: {0}")]
    ContactNotFound(String),

    #[error("Recovery attempt limit exceeded")]
    AttemptLimitExceeded,

    #[error("Invalid recovery shard data")]
    InvalidShardData,

    #[error("Reconstruction failed: {0}")]
    ReconstructionFailed(String),
}   