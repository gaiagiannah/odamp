//! Time-locked recovery: emergency access after a waiting period.
//!
//! If a user loses all devices and contacts, a pre-designated
//! emergency key can be unlocked after N days (configurable, default 30).
//! This prevents impulsive recovery while providing a last-resort option.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::error::RecoveryError;

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeLockConfig {
    pub lock_duration_days: u32,
    pub emergency_key_hash: Vec<u8>,
    pub activated_at: Option<DateTime<Utc>>,
    pub activated_by: String, // user confirmation hash
}

impl TimeLockConfig {
    pub fn activate(&mut self) {
        self.activated_at = Some(Utc::now());
    }

    pub fn is_unlocked(&self) -> bool {
        match self.activated_at {
            None => false,
            Some(activated) => {
                let unlock_time = activated + chrono::Duration::days(self.lock_duration_days as i64);
                Utc::now() >= unlock_time
            }
        }
    }

    pub fn time_remaining(&self) -> Option<chrono::Duration> {
        match self.activated_at {
            None => None,
            Some(activated) => {
                let unlock_time = activated + chrono::Duration::days(self.lock_duration_days as i64);
                if Utc::now() < unlock_time {
                    Some(unlock_time - Utc::now())
                } else {
                    None
                }
            }
        }
    }

    pub fn attempt_recovery(&self) -> Result<(), RecoveryError> {
        if !self.is_unlocked() {
            return Err(RecoveryError::TimeLockActive);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_lock_not_yet_unlocked() {
        let mut config = TimeLockConfig {
            lock_duration_days: 30,
            emergency_key_hash: vec![0u8; 32],
            activated_at: None,
            activated_by: "user_hash".into(),
        };
        config.activate();

        assert!(!config.is_unlocked());
        assert!(config.attempt_recovery().is_err());
    }
}   