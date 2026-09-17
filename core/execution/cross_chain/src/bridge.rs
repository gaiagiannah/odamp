//! Bridge abstraction: unified interface for all cross-chain bridges.

use serde::{Deserialize, Serialize};

/// Bridge protocol type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BridgeType {
    LayerZero,
    Wormhole,
    Axelar,
    CircleCctp,
    Stargate,
    NearIntents,
}

/// A cross-chain bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bridge {
    pub id: String,
    pub name: String,
    pub bridge_type: BridgeType,
    pub supported_chains: Vec<String>,
    pub base_fee_usd: f64,
    pub max_transfer_usd: f64,
    pub min_transfer_usd: f64,
    pub avg_time_secs: u64,
    pub security_score: f64,      // 0-100 (audit status, TVL, exploit history)
    pub tvl_usd: f64,
    pub is_operational: bool,
    pub has_active_exploit: bool,
}

impl Bridge {
    pub fn layer_zero() -> Self {
        Self {
            id: "layerzero".into(),
            name: "LayerZero".into(),
            bridge_type: BridgeType::LayerZero,
            supported_chains: vec![
                "ethereum".into(), "arbitrum".into(), "base".into(),
                "polygon".into(), "optimism".into(), "bsc".into(),
                "avalanche".into(), "solana".into(), "sui".into(),
                "aptos".into(), "ton".into(),
            ],
            base_fee_usd: 1.5,
            max_transfer_usd: 5_000_000.0,
            min_transfer_usd: 1.0,
            avg_time_secs: 300,
            security_score: 92.0,
            tvl_usd: 2_500_000_000.0,
            is_operational: true,
            has_active_exploit: false,
        }
    }

    pub fn wormhole() -> Self {
        Self {
            id: "wormhole".into(),
            name: "Wormhole".into(),
            bridge_type: BridgeType::Wormhole,
            supported_chains: vec![
                "ethereum".into(), "arbitrum".into(), "base".into(),
                "polygon".into(), "solana".into(), "sui".into(),
                "aptos".into(), "avalanche".into(),
            ],
            base_fee_usd: 2.0,
            max_transfer_usd: 10_000_000.0,
            min_transfer_usd: 1.0,
            avg_time_secs: 180,
            security_score: 85.0,
            tvl_usd: 1_800_000_000.0,
            is_operational: true,
            has_active_exploit: false,
        }
    }

    pub fn axelar() -> Self {
        Self {
            id: "axelar".into(),
            name: "Axelar".into(),
            bridge_type: BridgeType::Axelar,
            supported_chains: vec![
                "ethereum".into(), "arbitrum".into(), "base".into(),
                "polygon".into(), "solana".into(), "sui".into(),
                "cosmos".into(), "near".into(),
            ],
            base_fee_usd: 1.8,
            max_transfer_usd: 5_000_000.0,
            min_transfer_usd: 1.0,
            avg_time_secs: 420,
            security_score: 88.0,
            tvl_usd: 1_200_000_000.0,
            is_operational: true,
            has_active_exploit: false,
        }
    }

    pub fn circle_cctp() -> Self {
        Self {
            id: "circle-cctp".into(),
            name: "Circle CCTP V2".into(),
            bridge_type: BridgeType::CircleCctp,
            supported_chains: vec![
                "ethereum".into(), "arbitrum".into(), "base".into(),
                "solana".into(), "polygon".into(), "optimism".into(),
            ],
            base_fee_usd: 0.5,
            max_transfer_usd: 10_000_000.0,
            min_transfer_usd: 1.0,
            avg_time_secs: 120,
            security_score: 97.0,
            tvl_usd: 500_000_000.0,
            is_operational: true,
            has_active_exploit: false,
        }
    }

    pub fn stargate() -> Self {
        Self {
            id: "stargate".into(),
            name: "Stargate Finance".into(),
            bridge_type: BridgeType::Stargate,
            supported_chains: vec![
                "ethereum".into(), "arbitrum".into(), "base".into(),
                "polygon".into(), "optimism".into(), "avalanche".into(),
            ],
            base_fee_usd: 1.2,
            max_transfer_usd: 2_000_000.0,
            min_transfer_usd: 1.0,
            avg_time_secs: 240,
            security_score: 90.0,
            tvl_usd: 800_000_000.0,
            is_operational: true,
            has_active_exploit: false,
        }
    }
}

/// A quote from a bridge for a specific transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeQuote {
    pub bridge_id: String,
    pub from_chain: String,
    pub to_chain: String,
    pub token: String,
    pub amount: f64,
    pub amount_received: f64,
    pub fee_usd: f64,
    pub estimated_time_secs: u64,
    pub slippage_bps: f64,
    pub gas_cost_usd: f64,
    pub is_guaranteed: bool,
    pub security_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl BridgeQuote {
    pub fn total_cost_usd(&self) -> f64 {
        self.fee_usd + self.gas_cost_usd
    }

    pub fn net_received(&self) -> f64 {
        self.amount_received
    }
}   