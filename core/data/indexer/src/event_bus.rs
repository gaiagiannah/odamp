//! Event bus for propagating indexer events to downstream consumers
//! (portfolio engine, AI agents, notification service).

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::chain::ChainId;

/// Events emitted by the indexer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexerEvent {
    /// New block processed on a chain.
    NewBlock {
        chain: ChainId,
        block_number: u64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Token transfer detected.
    TokenTransfer {
        chain: ChainId,
        from: String,
        to: String,
        token: String,
        amount: f64,
        tx_hash: String,
    },

    /// DeFi protocol interaction detected.
    ProtocolInteraction {
        chain: ChainId,
        protocol: String,
        action: String,
        wallet: String,
        details: serde_json::Value,
    },

    /// Position opened or updated.
    PositionUpdated {
        chain: ChainId,
        wallet: String,
        position_id: uuid::Uuid,
        asset_symbol: String,
        new_amount: f64,
    },

    /// Chain reorganization detected.
    Reorg {
        chain: ChainId,
        old_block: u64,
        new_block: u64,
    },

    /// Indexer health status.
    Health {
        chain: ChainId,
        status: IndexerStatus,
        lag_blocks: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndexerStatus {
    Syncing,
    Healthy,
    Degraded,
    Stopped,
}

/// Broadcast channel for indexer events.
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<IndexerEvent>,
}

impl EventBus {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, _) = broadcast::channel(buffer_size);
        Self { tx }
    }

    pub fn publish(&self, event: IndexerEvent) -> Result<usize, broadcast::error::SendError<IndexerEvent>> {
        self.tx.send(event)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<IndexerEvent> {
        self.tx.subscribe()
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_pub_sub() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.publish(IndexerEvent::NewBlock {
            chain: ChainId::Ethereum,
            block_number: 19_000_001,
            timestamp: chrono::Utc::now(),
        }).unwrap();

        let event = rx.recv().await.unwrap();
        match event {
            IndexerEvent::NewBlock { block_number, .. } => {
                assert_eq!(block_number, 19_000_001);
            }
            _ => panic!("Wrong event type"),
        }
    }
}   