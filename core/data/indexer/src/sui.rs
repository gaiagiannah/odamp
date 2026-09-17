//! Sui chain indexing.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::chain::ChainConfig;
use crate::error::IndexerError;
use crate::positions::Position;

/// Parsed Sui transaction event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiEvent {
    pub digest: String,
    pub from: String,
    pub coin_transfers: Vec<SuiCoinTransfer>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiCoinTransfer {
    pub coin_type: String,
    pub symbol: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub decimals: u8,
}

/// Sui indexer.
pub struct SuiIndexer {
    chain_config: Arc<ChainConfig>,
    watched_addresses: Vec<String>,
}

impl SuiIndexer {
    pub fn new(chain_config: Arc<ChainConfig>) -> Self {
        Self {
            chain_config,
            watched_addresses: vec![],
        }
    }

    pub fn watch_address(&mut self, address: String) {
        if !self.watched_addresses.contains(&address) {
            self.watched_addresses.push(address);
        }
    }

    pub async fn start(&mut self) -> Result<(), IndexerError> {
        tracing::info!("Starting Sui indexer");
        // Production: connect via Sui JSON-RPC
        // Subscribe to object changes for watched addresses
        Ok(())
    }

    pub fn process_event(&mut self, event: &SuiEvent) -> Vec<Position> {
        event
            .coin_transfers
            .iter()
            .map(|t| Position {
                position_id: uuid::Uuid::new_v4(),
                chain: self.chain_config.chain_id,
                wallet_address: t.to.clone(),
                asset_type: crate::positions::AssetType::FungibleToken,
                asset_id: t.coin_type.clone(),
                asset_symbol: t.symbol.clone(),
                amount: (t.amount as f64) / (10f64.powi(t.decimals as i32)),
                raw_amount: t.amount as u128,
                protocol: None,
                protocol_sub_type: None,
                opened_at: event.timestamp,
                updated_at: chrono::Utc::now(),
            })
            .collect()
    }
}   