//! Chain identification and configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Unique chain identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ChainId {
    // EVM chains
    Ethereum = 1,
    Arbitrum = 42161,
    Base = 8453,
    Optimism = 10,
    Polygon = 137,
    Bsc = 56,
    Avalanche = 43114,
    Gnosis = 100,
    Fantom = 250,
    Celo = 42220,

    // Non-EVM
    Solana = 1000001,
    Sui = 1000002,
    Aptos = 1000003,
    TON = 1000004,

    // Testnets
    Sepolia = 11155111,
    ArbitrumSepolia = 421614,
    BaseSepolia = 84532,
    SolanaDevnet = 10000011,
    SuiTestnet = 10000021,
}

impl ChainId {
    pub fn name(&self) -> &'static str {
        match self {
            ChainId::Ethereum => "ethereum",
            ChainId::Arbitrum => "arbitrum",
            ChainId::Base => "base",
            ChainId::Optimism => "optimism",
            ChainId::Polygon => "polygon",
            ChainId::Bsc => "bsc",
            ChainId::Avalanche => "avalanche",
            ChainId::Gnosis => "gnosis",
            ChainId::Fantom => "fantom",
            ChainId::Celo => "celo",
            ChainId::Solana => "solana",
            ChainId::Sui => "sui",
            ChainId::Aptos => "aptos",
            ChainId::TON => "ton",
            ChainId::Sepolia => "sepolia",
            ChainId::ArbitrumSepolia => "arbitrum-sepolia",
            ChainId::BaseSepolia => "base-sepolia",
            ChainId::SolanaDevnet => "solana-devnet",
            ChainId::SuiTestnet => "sui-testnet",
        }
    }

    pub fn is_evm(&self) -> bool {
        matches!(
            self,
            ChainId::Ethereum
                | ChainId::Arbitrum
                | ChainId::Base
                | ChainId::Optimism
                | ChainId::Polygon
                | ChainId::Bsc
                | ChainId::Avalanche
                | ChainId::Gnosis
                | ChainId::Fantom
                | ChainId::Celo
                | ChainId::Sepolia
                | ChainId::ArbitrumSepolia
                | ChainId::BaseSepolia
        )
    }

    pub fn is_testnet(&self) -> bool {
        matches!(
            self,
            ChainId::Sepolia
                | ChainId::ArbitrumSepolia
                | ChainId::BaseSepolia
                | ChainId::SolanaDevnet
                | ChainId::SuiTestnet
        )
    }

    pub fn native_token(&self) -> &'static str {
        match self {
            ChainId::Ethereum | ChainId::Arbitrum | ChainId::Base | ChainId::Optimism
            | ChainId::Sepolia | ChainId::ArbitrumSepolia | ChainId::BaseSepolia => "ETH",
            ChainId::Polygon => "POL",
            ChainId::Bsc => "BNB",
            ChainId::Avalanche => "AVAX",
            ChainId::Gnosis => "xDAI",
            ChainId::Fantom => "FTM",
            ChainId::Celo => "CELO",
            ChainId::Solana | ChainId::SolanaDevnet => "SOL",
            ChainId::Sui | ChainId::SuiTestnet => "SUI",
            ChainId::Aptos => "APT",
            ChainId::TON => "TON",
        }
    }

    pub fn decimals(&self) -> u8 {
        match self {
            ChainId::Solana | ChainId::SolanaDevnet => 9,
            ChainId::Sui | ChainId::SuiTestnet => 9,
            _ => 18,
        }
    }
}

/// Configuration for connecting to a specific chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: ChainId,
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub explorer_url: String,
    pub block_time_ms: u64,
    pub max_reorg_depth: u64,
    pub enabled: bool,
}

impl ChainConfig {
    pub fn ethereum_mainnet() -> Self {
        Self {
            chain_id: ChainId::Ethereum,
            rpc_url: "https://eth.llamarpc.com".into(),
            ws_url: Some("wss://eth.llamarpc.com".into()),
            explorer_url: "https://etherscan.io".into(),
            block_time_ms: 12_000,
            max_reorg_depth: 2,
            enabled: true,
        }
    }

    pub fn arbitrum_mainnet() -> Self {
        Self {
            chain_id: ChainId::Arbitrum,
            rpc_url: "https://arb1.arbitrum.io/rpc".into(),
            ws_url: Some("wss://arb1.arbitrum.io/rpc".into()),
            explorer_url: "https://arbiscan.io".into(),
            block_time_ms: 250,
            max_reorg_depth: 1,
            enabled: true,
        }
    }

    pub fn base_mainnet() -> Self {
        Self {
            chain_id: ChainId::Base,
            rpc_url: "https://mainnet.base.org".into(),
            ws_url: Some("wss://mainnet.base.org".into()),
            explorer_url: "https://basescan.org".into(),
            block_time_ms: 2_000,
            max_reorg_depth: 1,
            enabled: true,
        }
    }

    pub fn solana_mainnet() -> Self {
        Self {
            chain_id: ChainId::Solana,
            rpc_url: "https://api.mainnet-beta.solana.com".into(),
            ws_url: Some("wss://api.mainnet-beta.solana.com".into()),
            explorer_url: "https://solscan.io".into(),
            block_time_ms: 400,
            max_reorg_depth: 0,
            enabled: true,
        }
    }

    pub fn sui_mainnet() -> Self {
        Self {
            chain_id: ChainId::Sui,
            rpc_url: "https://fullnode.mainnet.sui.io".into(),
            ws_url: None,
            explorer_url: "https://suiscan.xyz".into(),
            block_time_ms: 400,
            max_reorg_depth: 0,
            enabled: true,
        }
    }
}

/// Registry of all configured chains.
#[derive(Debug, Clone)]
pub struct ChainRegistry {
    chains: HashMap<ChainId, Arc<ChainConfig>>,
}

impl ChainRegistry {
    pub fn new() -> Self {
        Self { chains: HashMap::new() }
    }

    pub fn register(&mut self, config: ChainConfig) {
        self.chains.insert(config.chain_id, Arc::new(config));
    }

    pub fn get(&self, chain_id: ChainId) -> Option<Arc<ChainConfig>> {
        self.chains.get(&chain_id).cloned()
    }

    pub fn enabled_chains(&self) -> Vec<Arc<ChainConfig>> {
        self.chains
            .values()
            .filter(|c| c.enabled)
            .cloned()
            .collect()
    }

    pub fn default_mainnet_registry() -> Self {
        let mut registry = Self::new();
        registry.register(ChainConfig::ethereum_mainnet());
        registry.register(ChainConfig::arbitrum_mainnet());
        registry.register(ChainConfig::base_mainnet());
        registry.register(ChainConfig::solana_mainnet());
        registry.register(ChainConfig::sui_mainnet());
        registry
    }
}

impl Default for ChainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_registry() {
        let registry = ChainRegistry::default_mainnet_registry();
        let enabled = registry.enabled_chains();
        assert_eq!(enabled.len(), 5);
        assert!(registry.get(ChainId::Ethereum).is_some());
        assert!(registry.get(ChainId::Solana).is_some());
    }

    #[test]
    fn test_chain_properties() {
        assert!(ChainId::Ethereum.is_evm());
        assert!(!ChainId::Solana.is_evm());
        assert_eq!(ChainId::Ethereum.decimals(), 18);
        assert_eq!(ChainId::Solana.decimals(), 9);
        assert!(!ChainId::Ethereum.is_testnet());
        assert!(ChainId::Sepolia.is_testnet());
    }
}   