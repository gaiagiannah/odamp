//! Social recovery: trusted contacts hold recovery shares.
//! User designates N trusted contacts; T of them can initiate recovery.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::RecoveryError;

#[derive(Debug, Serialize, Deserialize)]
pub struct TrustedContact {
    pub contact_id: Uuid,
    pub address: String, // email or encrypted contact identifier
    pub share_id: u32,
    pub added_at: chrono::DateTime<chrono::Utc>,
    pub last_verified: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocialRecoveryConfig {
    pub threshold: u32,
    pub contacts: Vec<TrustedContact>,
    pub cooldown_hours: u32, // minimum time between recovery attempts
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecoveryRequest {
    pub request_id: Uuid,
    pub initiated_by: Uuid, // contact_id
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub shares_submitted: Vec<u32>,
    pub status: RecoveryStatus,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum RecoveryStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
    Expired,
}

impl SocialRecoveryConfig {
    pub fn is_threshold_met(&self, submitted_shares: &[u32]) -> bool {
        let valid: Vec<u32> = submitted_shares
            .iter()
            .filter(|s| self.contacts.iter().any(|c| c.share_id == **s))
            .copied()
            .collect();
        valid.len() as u32 >= self.threshold
    }

    pub fn add_contact(&mut self, contact: TrustedContact) -> Result<(), RecoveryError> {
        if self.contacts.iter().any(|c| c.address == contact.address) {
            return Err(RecoveryError::DuplicateShare(contact.share_id));
        }
        self.contacts.push(contact);
        Ok(())
    }

    pub fn remove_contact(&mut self, contact_id: Uuid) -> Result<(), RecoveryError> {
        if self.contacts.len() <= self.threshold as usize {
            return Err(RecoveryError::InsufficientShares {
                have: self.contacts.len() as u32,
                need: self.threshold,
            });
        }
        self.contacts.retain(|c| c.contact_id != contact_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_recovery_threshold() {
        let mut config = SocialRecoveryConfig {
            threshold: 2,
            contacts: vec![],
            cooldown_hours: 24,
        };

        for i in 1..=3 {
            config.add_contact(TrustedContact {
                contact_id: Uuid::new_v4(),
                address: format!("contact{}@example.com", i),
                share_id: i,
                added_at: chrono::Utc::now(),
                last_verified: chrono::Utc::now(),
            }).unwrap();
        }

        assert!(!config.is_threshold_met(&[1]));
        assert!(config.is_threshold_met(&[1, 2]));
        assert!(config.is_threshold_met(&[2, 3]));
    }
}   