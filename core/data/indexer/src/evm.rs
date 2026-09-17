//! EVM chain indexing: Ethereum, Arbitrum, Base, Optimism, Polygon, etc.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::chain::{ChainConfig, ChainId};
use crate::error::IndexerError;
use crate::positions::Position;

/// Parsed EVM transaction event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmEvent {
    pub tx_hash: String,
    pub block_number: u64,
    pub from: String,
    pub to: Option<String>,
    pub value_wei: u128,
    pub token_transfers: Vec<TokenTransfer>,
    pub protocol_interactions: Vec<ProtocolInteraction>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    pub token_address: String,
    pub token_symbol: String,
    pub from: String,
    pub to: String,
    pub amount: u128,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolInteraction {
    pub protocol: String,
    pub action: String,
    pub details: serde_json::Value,
}

/// EVM indexer for a single chain.
pub struct EvmIndexer {
    chain_config: Arc<ChainConfig>,
    current_block: u64,
    watched_addresses: Vec<String>,
}

impl EvmIndexer {
    pub fn new(chain_config: Arc<ChainConfig>) -> Self {
        Self {
            chain_config,
            current_block: 0,
            watched_addresses: vec![],
        }
    }

    pub fn watch_address(&mut self, address: String) {
        if !self.watched_addresses.contains(&address) {
            self.watched_addresses.push(address);
        }
    }

    pub fn current_block(&self) -> u64 {
        self.current_block
    }

    /// Connects to the chain and begins indexing from the latest block.
    pub async fn start(&mut self) -> Result<(), IndexerError> {
        tracing::info!(
            chain = self.chain_config.chain_id.name(),
            "Starting EVM indexer"
        );

        // Production: connect via alloy/ethers WebSocket
        // 1. Get latest block number
        // 2. Subscribe to new headers
        // 3. For each new block, fetch logs for watched addresses
        // 4. Parse Transfer events, protocol-specific events
        // 5. Update positions

        self.current_block = self.get_latest_block().await?;
        Ok(())
    }

    /// Processes a single block's events.
    pub fn process_block(&mut self, events: Vec<EvmEvent>) -> Vec<Position> {
        let mut new_positions = Vec::new();

        for event in &events {
            // Check if any watched address is involved
            let is_relevant = self.watched_addresses.iter().any(|addr| {
                event.from.to_lowercase().contains(&addr.to_lowercase())
                    || event.token_transfers.iter().any(|t| {
                        t.from.to_lowercase().contains(&addr.to_lowercase())
                            || t.to.to_lowercase().contains(&addr.to_lowercase())
                    })
            });

            if !is_relevant {
                continue;
            }

            for transfer in &event.token_transfers {
                let pos = Position {
                    position_id: uuid::Uuid::new_v4(),
                    chain: self.chain_config.chain_id,
                    wallet_address: transfer.to.clone(),
                    asset_type: crate::positions::AssetType::FungibleToken,
                    asset_id: transfer.token_address.clone(),
                    asset_symbol: transfer.token_symbol.clone(),
                    amount: (transfer.amount as f64) / (10f64.powi(transfer.decimals as i32)),
                    raw_amount: transfer.amount,
                    protocol: None,
                    protocol_sub_type: None,
                    opened_at: event.timestamp,
                    updated_at: chrono::Utc::now(),
                };
                new_positions.push(pos);
            }

            for interaction in &event.protocol_interactions {
                let pos = Position {
                    position_id: uuid::Uuid::new_v4(),
                    chain: self.chain_config.chain_id,
                    wallet_address: event.from.clone(),
                    asset_type: crate::positions::AssetType::DefiPosition,
                    asset_id: interaction.protocol.clone(),
                    asset_symbol: format!("{}-{}", interaction.protocol, interaction.action),
                    amount: 0.0, // filled by parser
                    raw_amount: 0,
                    protocol: Some(interaction.protocol.clone()),
                    protocol_sub_type: Some(interaction.action.clone()),
                    opened_at: event.timestamp,
                    updated_at: chrono::Utc::now(),
                };
                new_positions.push(pos);
            }
        }

        self.current_block += 1;
        new_positions
    }

    async fn get_latest_block(&self) -> Result<u64, IndexerError> {
        // Production: RPC call to eth_blockNumber
        // Placeholder for development
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evm_indexer_watches() {
        let config = Arc::new(ChainConfig::ethereum_mainnet());
        let mut indexer = EvmIndexer::new(config);
        indexer.watch_address("0x1234567890abcdef1234567890abcdef12345678".into());
        assert_eq!(indexer.watched_addresses.len(), 1);
    }
}   