//! Solana chain indexing.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::chain::ChainConfig;
use crate::error::IndexerError;
use crate::positions::Position;

/// Parsed Solana transaction event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaEvent {
    pub signature: String,
    pub slot: u64,
    pub from: String,
    pub token_transfers: Vec<SolanaTokenTransfer>,
    pub protocol_interactions: Vec<crate::evm::ProtocolInteraction>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaTokenTransfer {
    pub mint: String,
    pub symbol: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub decimals: u8,
}

/// Solana indexer.
pub struct SolanaIndexer {
    chain_config: Arc<ChainConfig>,
    current_slot: u64,
    watched_addresses: Vec<String>,
}

impl SolanaIndexer {
    pub fn new(chain_config: Arc<ChainConfig>) -> Self {
        Self {
            chain_config,
            current_slot: 0,
            watched_addresses: vec![],
        }
    }

    pub fn watch_address(&mut self, address: String) {
        if !self.watched_addresses.contains(&address) {
            self.watched_addresses.push(address);
        }
    }

    pub async fn start(&mut self) -> Result<(), IndexerError> {
        tracing::info!("Starting Solana indexer");
        // Production: connect via solana-client WebSocket
        // Subscribe to account changes for watched addresses
        // Parse SPL Token transfers, protocol CPIs
        Ok(())
    }

    pub fn process_event(&mut self, event: &SolanaEvent) -> Vec<Position> {
        let mut positions = Vec::new();

        for transfer in &event.token_transfers {
            let pos = Position {
                position_id: uuid::Uuid::new_v4(),
                chain: self.chain_config.chain_id,
                wallet_address: transfer.to.clone(),
                asset_type: crate::positions::AssetType::FungibleToken,
                asset_id: transfer.mint.clone(),
                asset_symbol: transfer.symbol.clone(),
                amount: (transfer.amount as f64) / (10f64.powi(transfer.decimals as i32)),
                raw_amount: transfer.amount as u128,
                protocol: None,
                protocol_sub_type: None,
                opened_at: event.timestamp,
                updated_at: chrono::Utc::now(),
            };
            positions.push(pos);
        }

        self.current_slot += 1;
        positions
    }
}   