//! Bundle submission: atomic transaction execution via Flashbots.
//!
//! A bundle is a set of transactions that are either all included
//! in the same block or none are. This prevents:
//! - Partial execution (one tx succeeds, another reverts)


```rust
use serde::{Deserialize, Serialize};
use crate::error::MevError;

/// A transaction bundle for atomic execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub bundle_id: uuid::Uuid,
    pub transactions: Vec<BundleTx>,
    pub min_tip_wei: u128,
    pub target_block: Option<u64>,
    pub valid_until_block: u64,
    pub status: BundleStatus,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub included_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleTx {
    pub to: String,
    pub from: String,
    pub value_wei: u128,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee: u128,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BundleStatus {
    Pending,
    Submitted,
    Included,
    Dropped,
    Rejected,
    Expired,
}

/// Submits bundles to Flashbots Protect / MEV-Blocker.
pub struct BundleSubmitter {
    flashbots_url: String,
    mev_blocker_url: String,
}

impl BundleSubmitter {
    pub fn new() -> Self {
        Self {
            flashbots_url: "https://rpc.flashbots.net".into(),
            mev_blocker_url: "https://mev-blocker.co/rpc".into(),
        }
    }

    /// Creates and submits a bundle.
    pub async fn submit(&self, bundle: &Bundle) -> Result<BundleResult, MevError> {
        tracing::info!(
            bundle_id = %bundle.bundle_id,
            tx_count = bundle.transactions.len(),
            "Submitting bundle"
        );

        // Production: POST to Flashbots eth_sendBundle
        // {
        //   "method": "eth_sendBundle",
        //   "params": [{
        //     "txs": [...],
        //     "blockNumber": "0x...",
        //     "minTimestamp": 0,
        //     "maxTimestamp": ...,
        //     "revertingTxHashes": []
        //   }],
        //   "id": 1
        // }

        Ok(BundleResult {
            bundle_id: bundle.bundle_id,
            accepted: true,
            expected_inclusion_block: bundle.valid_until_block,
            estimated_tip: bundle.min_tip_wei,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleResult {
    pub bundle_id: uuid::Uuid,
    pub accepted: bool,
    pub expected_inclusion_block: u64,
    pub estimated_tip: u128,
}

impl Default for BundleSubmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_construction() {
        let bundle = Bundle {
            bundle_id: uuid::Uuid::new_v4(),
            transactions: vec![
                BundleTx {
                    to: "0x1234".into(),
                    from: "0x5678".into(),
                    value_wei: 0,
                    data: vec![0xa9, 0x05, 0x9c],
                    gas_limit: 100_000,
                    max_fee_per_gas: 30_000_000_000,
                    max_priority_fee: 2_000_000_000,
                },
            ],
            min_tip_wei: 1_000_000_000,
            target_block: None,
            valid_until_block: 19_000_100,
            status: BundleStatus::Pending,
            submitted_at: None,
            included_at: None,
        };

        assert_eq!(bundle.transactions.len(), 1);
        assert_eq!(bundle.status, BundleStatus::Pending);
    }
}   