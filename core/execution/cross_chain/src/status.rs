//! Transfer status tracking: monitors cross-chain transfers from
//! source confirmation to destination receipt.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::intent::TransferStatus;
use crate::error::CrossChainError;

/// Status of an in-flight cross-chain transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferStatusUpdate {
    pub transfer_id: uuid::Uuid,
    pub bridge_id: String,
    pub from_chain: String,
    pub to_chain: String,
    pub token: String,
    pub amount: f64,
    pub status: TransferStatus,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub source_confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub destination_received_at: Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_completion: chrono::DateTime<chrono::Utc>,
    pub is_stuck: bool,
    pub stuck_reason: Option<String>,
}

/// Tracks all in-flight transfers.
pub struct TransferTracker {
    transfers: HashMap<uuid::Uuid, TransferStatusUpdate>,
    stuck_threshold_secs: u64,
}

impl TransferTracker {
    pub fn new(stuck_threshold_secs: u64) -> Self {
        Self {
            transfers: HashMap::new(),
            stuck_threshold_secs,
        }
    }

    pub fn register(&mut self, transfer: TransferStatusUpdate) {
        self.transfers.insert(transfer.transfer_id, transfer);
    }

    pub fn update_status(
        &mut self,
        transfer_id: uuid::Uuid,
        status: TransferStatus,
    ) -> Result<(), CrossChainError> {
        let transfer = self.transfers.get_mut(&transfer_id)
            .ok_or_else(|| CrossChainError::TransferFailed("Not found".into()))?;

        transfer.status = status.clone();

        match &status {
            TransferStatus::SourceConfirmed => {
                transfer.source_confirmed_at = Some(chrono::Utc::now());
            }
            TransferStatus::Complete => {
                transfer.destination_received_at = Some(chrono::Utc::now());
                transfer.is_stuck = false;
            }
            TransferStatus::Failed | TransferStatus::Stuck => {
                transfer.is_stuck = true;
            }
            _ => {}
        }

        Ok(())
    }

    /// Checks for stuck transfers and flags them.
    pub fn check_stuck(&mut self) -> Vec<uuid::Uuid> {
        let now = chrono::Utc::now();
        let mut stuck = vec![];

        for (id, transfer) in self.transfers.iter_mut() {
            if matches!(transfer.status, TransferStatus::Bridging | TransferStatus::DestinationPending) {
                let elapsed = now.signed_duration_since(transfer.estimated_completion);
                if elapsed > chrono::Duration::seconds(self.stuck_threshold_secs as i64) {
                    transfer.is_stuck = true;
                    transfer.stuck_reason = Some(format!(
                        "No progress after {}s past estimated completion",
                        elapsed.num_seconds()
                    ));
                    transfer.status = TransferStatus::Stuck;
                    stuck.push(*id);
                }
            }
        }

        stuck
    }

    pub fn active_transfers(&self) -> Vec<&TransferStatusUpdate> {
        self.transfers
            .values()
            .filter(|t| !matches!(t.status, TransferStatus::Complete | TransferStatus::Failed))
            .collect()
    }

    pub fn get(&self, id: &uuid::Uuid) -> Option<&TransferStatusUpdate> {
        self.transfers.get(id)
    }
}

impl Default for TransferTracker {
    fn default() -> Self {
        Self::new(900) // 15 min stuck threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_lifecycle() {
        let mut tracker = TransferTracker::default();

        let transfer = TransferStatusUpdate {
            transfer_id: uuid::Uuid::new_v4(),
            bridge_id: "circle-cctp".into(),
            from_chain: "ethereum".into(),
            to_chain: "solana".into(),
            token: "USDC".into(),
            amount: 5_000.0,
            status: TransferStatus::Pending,
            source_tx_hash: None,
            destination_tx_hash: None,
            source_confirmed_at: None,
            destination_received_at: None,
            estimated_completion: chrono::Utc::now() + chrono::Duration::seconds(120),
            is_stuck: false,
            stuck_reason: None,
        };

        let id = transfer.transfer_id;
        tracker.register(transfer);

        tracker.update_status(id, TransferStatus::SourceConfirmed).unwrap();
        tracker.update_status(id, TransferStatus::Bridging).unwrap();
        tracker.update_status(id, TransferStatus::Complete).unwrap();

        let final_state = tracker.get(&id).unwrap();
        assert_eq!(final_state.status, TransferStatus::Complete);
        assert!(final_state.destination_received_at.is_some());
    }
}   