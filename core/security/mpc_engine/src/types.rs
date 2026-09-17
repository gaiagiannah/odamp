//! Common types for the MPC engine.

use serde::{Deserialize, Serialize};

/// Represents a wallet identity across chains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletIdentity {
    pub wallet_id: uuid::Uuid,
    pub address: String,
    pub chain: String,
    pub public_key: Vec<u8>,
    pub security_tier: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Transaction to be signed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignableTransaction {
    pub chain: String,
    pub to: String,
    pub from: String,
    pub value: String,
    pub data: Vec<u8>,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<String>,
    pub nonce: Option<u64>,
    /// The computed hash to sign (chain-specific)
    pub message_hash: [u8; 32],
}   