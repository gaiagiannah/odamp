//! DeFi protocol interaction parser.
//!
//! Recognizes and parses events from major DeFi protocols
//! to classify positions correctly.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Known DeFi protocol identifiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Protocol {
    AaveV3,
    Morpho,
    CompoundV3,
    YearnV3,
    UniswapV3,
    Curve,
    Lido,
    RocketPool,
    Balancer,
    MakerDao,
    AaveV2,
    CompoundV2,
    SushiSwap,
    PancakeSwap,
    Raydium,
    Orca,
    Marinade,
    // Tokenized RWA
    Securitize,
    OndoFinance,
    Centrifuge,
    MetalsIo,
    ReElement,
    // Tokenized Equities
    XStocks,
    Backed,
}

impl Protocol {
    pub fn name(&self) -> &'static str {
        match self {
            Protocol::AaveV3 => "aave-v3",
            Protocol::Morpho => "morpho",
            Protocol::CompoundV3 => "compound-v3",
            Protocol::YearnV3 => "yearn-v3",
            Protocol::UniswapV3 => "uniswap-v3",
            Protocol::Curve => "curve",
            Protocol::Lido => "lido",
            Protocol::RocketPool => "rocketpool",
            Protocol::Balancer => "balancer",
            Protocol::MakerDao => "makerdao",
            Protocol::AaveV2 => "aave-v2",
            Protocol::CompoundV2 => "compound-v2",
            Protocol::SushiSwap => "sushiswap",
            Protocol::PancakeSwap => "pancakeswap",
            Protocol::Raydium => "raydium",
            Protocol::Orca => "orca",
            Protocol::Marinade => "marinade",
            Protocol::Securitize => "securitize",
            Protocol::OndoFinance => "ondo-finance",
            Protocol::Centrifuge => "centrifuge",
            Protocol::MetalsIo => "metals-io",
            Protocol::ReElement => "reelement",
            Protocol::XStocks => "xstocks",
            Protocol::Backed => "backed",
        }
    }

    pub fn category(&self) -> ProtocolCategory {
        match self {
            Protocol::AaveV3 | Protocol::Morpho | Protocol::CompoundV3
            | Protocol::AaveV2 | Protocol::CompoundV2 | Protocol::MakerDao => {
                ProtocolCategory::Lending
            }
            Protocol::YearnV3 => ProtocolCategory::Yield,
            Protocol::UniswapV3 | Protocol::Curve | Protocol::Balancer
            | Protocol::SushiSwap | Protocol::PancakeSwap | Protocol::Raydium
            | Protocol::Orca => ProtocolCategory::Dex,
            Protocol::Lido | Protocol::RocketPool | Protocol::Marinade => {
                ProtocolCategory::Staking
            }
            Protocol::Securitize | Protocol::OndoFinance | Protocol::Centrifuge
            | Protocol::MetalsIo | Protocol::ReElement => ProtocolCategory::Rwa,
            Protocol::XStocks | Protocol::Backed => ProtocolCategory::TokenizedEquity,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProtocolCategory {
    Lending,
    Yield,
    Dex,
    Staking,
    Rwa,
    TokenizedEquity,
}

/// A parsed DeFi position with protocol-specific details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefiPosition {
    pub protocol: Protocol,
    pub category: ProtocolCategory,
    pub sub_type: String,       // "supply", "borrow", "lp", "stake", "vault"
    pub underlying_assets: Vec<String>,
    pub amount: f64,
    pub apy: Option<f64>,       // current APY for yield positions
    pub health_factor: Option<f64>, // for lending positions
    pub risk_score: Option<f64>,   // 0-100 protocol risk
}

/// Maps contract addresses to known protocols.
/// In production, this is loaded from a JSON file and updated regularly.
pub struct ProtocolRegistry {
    evm_addresses: HashMap<String, Protocol>,
    solana_programs: HashMap<String, Protocol>,
    sui_packages: HashMap<String, Protocol>,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        let mut evm_addresses: HashMap<String, Protocol> = HashMap::new();

        // Aave V3 (Ethereum)
        evm_addresses.insert("0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2".into(), Protocol::AaveV3);
        // Uniswap V3 (Ethereum)
        evm_addresses.insert("0xe592427a0aece92de3edee1f18e0157c05861564".into(), Protocol::UniswapV3);
        // Lido (Ethereum)
        evm_addresses.insert("0x7d2768de32b0b80b7a3454c06bdac94a69ddc7a9".into(), Protocol::Lido);
        // Curve (Ethereum)
        evm_addresses.insert("0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7".into(), Protocol::Curve);
        // Ondo Finance
        evm_addresses.insert("0x75231f58b43240c9718d554723b82a92177586d4".into(), Protocol::OndoFinance);
        // Metals.io (Tezos - different registry)
        // Securitize
        evm_addresses.insert("0x1683a0b19475e05c8d370e617210306b24bfbc0e".into(), Protocol::Securitize);

        let mut solana_programs: HashMap<String, Protocol> = HashMap::new();
        solana_programs.insert("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".into(), Protocol::Orca);
        solana_programs.insert("LaMqVWxmEKxavY1Y134sPVTVzg4YgiXL1MEfGgF2qE5".into(), Protocol::Marinade);

        Self {
            evm_addresses,
            solana_programs,
            sui_packages: HashMap::new(),
        }
    }

    pub fn lookup_evm(&self, address: &str) -> Option<&Protocol> {
        self.evm_addresses.get(&address.to_lowercase())
    }

    pub fn lookup_solana(&self, program: &str) -> Option<&Protocol> {
        self.solana_programs.get(program)
    }
}

impl Default for ProtocolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_lookup() {
        let registry = ProtocolRegistry::new();
        let proto = registry.lookup_evm("0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2");
        assert_eq!(proto, Some(&Protocol::AaveV3));
        assert_eq!(Protocol::AaveV3.category(), ProtocolCategory::Lending);
    }

    #[test]
    fn test_protocol_categories() {
        assert_eq!(Protocol::UniswapV3.category(), ProtocolCategory::Dex);
        assert_eq!(Protocol::Lido.category(), ProtocolCategory::Staking);
        assert_eq!(Protocol::OndoFinance.category(), ProtocolCategory::Rwa);
        assert_eq!(Protocol::XStocks.category(), ProtocolCategory::TokenizedEquity);
    }
}   