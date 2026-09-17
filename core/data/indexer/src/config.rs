//! Indexer configuration.

use serde::{Deserialize, Serialize};
use crate::chain::{ChainConfig, ChainRegistry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerConfig {
    pub chains: Vec<ChainConfig>,
    pub poll_interval_ms: u64,
    pub batch_size: u32,
    pub max_concurrent_chains: u32,
    pub database_url: String,
    pub event_buffer_size: usize,
}

impl IndexerConfig {
    pub fn default_dev() -> Self {
        Self {
            chains: vec![
                ChainConfig::ethereum_mainnet(),
                ChainConfig::arbitrum_mainnet(),
                ChainConfig::base_mainnet(),
                ChainConfig::solana_mainnet(),
                ChainConfig::sui_mainnet(),
            ],
            poll_interval_ms: 2_000,
            batch_size: 100,
            max_concurrent_chains: 5,
            database_url: "postgres://localhost:5432/odamp".into(),
            event_buffer_size: 4096,
        }
    }

    pub fn registry(&self) -> ChainRegistry {
        let mut registry = ChainRegistry::new();
        for config in &self.chains {
            registry.register(config.clone());
        }
        registry
    }
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self::default_dev()
    }
}   