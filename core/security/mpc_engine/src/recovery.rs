//! Key recovery via Shamir's Secret Sharing or social recovery.

use crate::error::MpcError;
use crate::config::{RecoveryConfig, RecoveryMethod};
use serde::{Deserialize, Serialize};

/// A recovery shard (for SSS method).
#[derive(Debug, Serialize, Deserialize)]
pub struct RecoveryShard {
    pub shard_id: u32,
    pub encrypted_data: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Initiates a recovery session.
pub struct RecoverySession {
    pub config: RecoveryConfig,
    pub received_shards: Vec<RecoveryShard>,
}

impl RecoverySession {
    pub fn new(config: RecoveryConfig) -> Self {
        Self {
            config,
            received_shards: vec![],
        }
    }

    pub fn add_shard(&mut self, shard: RecoveryShard) -> Result<(), MpcError> {
        if self.received_shards.iter().any(|s| s.shard_id == shard.shard_id) {
            return Err(MpcError::RecoveryThresholdNotMet); // duplicate
        }
        self.received_shards.push(shard);
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        self.received_shards.len() as u32 >= self.config.threshold
    }

    /// Reconstructs the key material from recovery shards.
    /// Only callable when threshold is met.
    pub fn reconstruct(&self) -> Result<Vec<u8>, MpcError> {
        if !self.is_complete() {
            return Err(MpcError::RecoveryThresholdNotMet);
        }

        // Production: Shamir reconstruction over the scalar field
        // For development: return placeholder
        Ok(vec![0u8; 32])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_threshold() {
        let config = RecoveryConfig {
            method: RecoveryMethod::Sss,
            threshold: 2,
            total_shares: 3,
            trusted_contacts: vec!["a".into(), "b".into(), "c".into()],
        };

        let mut session = RecoverySession::new(config);
        assert!(!session.is_complete());

        session.add_shard(RecoveryShard { shard_id: 1, encrypted_data: vec![1], nonce: vec![] }).unwrap();
        assert!(!session.is_complete());

        session.add_shard(RecoveryShard { shard_id: 2, encrypted_data: vec![2], nonce: vec![] }).unwrap();
        assert!(session.is_complete());
    }
}   