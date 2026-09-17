//! Position tracking across all chains and protocols.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::chain::ChainId;

/// Asset type classification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssetType {
    /// Native token (ETH, SOL, SUI, etc.)
    Native,
    /// Fungible token (ERC-20, SPL, Sui coin)
    FungibleToken,
    /// Non-fungible token
    Nft,
    /// DeFi position (LP, lending, staking)
    DefiPosition,
    /// Tokenized real-world asset
    TokenizedRwa,
    /// Tokenized equity
    TokenizedEquity,
    /// Stablecoin
    Stablecoin,
}

/// A single position held by a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub position_id: uuid::Uuid,
    pub chain: ChainId,
    pub wallet_address: String,
    pub asset_type: AssetType,
    pub asset_id: String,        // token address or identifier
    pub asset_symbol: String,     // "ETH", "USDC", "PAXG", etc.
    pub amount: f64,              // human-readable amount
    pub raw_amount: u128,         // on-chain integer amount
    pub protocol: Option<String>, // "aave-v3", "uniswap-v3", "metals-io"
    pub protocol_sub_type: Option<String>, // "lending", "lp", "staking"
    pub opened_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Position {
    pub fn value_usd(&self, price_usd: f64) -> f64 {
        self.amount * price_usd
    }
}

/// Aggregated portfolio state for a single wallet.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletState {
    pub address: String,
    pub chain: ChainId,
    pub positions: Vec<Position>,
    pub total_value_usd: f64,
    pub last_synced_block: u64,
    pub last_synced_at: chrono::DateTime<chrono::Utc>,
}

impl WalletState {
    pub fn positions_by_type(&self) -> HashMap<&AssetType, Vec<&Position>> {
        let mut map: HashMap<&AssetType, Vec<&Position>> = HashMap::new();
        for pos in &self.positions {
            map.entry(&pos.asset_type).or_default().push(pos);
        }
        map
    }

    pub fn defi_exposure(&self) -> f64 {
        self.positions
            .iter()
            .filter(|p| p.asset_type == AssetType::DefiPosition)
            .map(|p| p.amount) // simplified: amount already in USD for DeFi
            .sum()
    }
}

/// Tracks positions across all wallets and chains for a user.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortfolioState {
    pub user_id: uuid::Uuid,
    pub wallets: Vec<WalletState>,
    pub total_value_usd: f64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl PortfolioState {
    pub fn new(user_id: uuid::Uuid) -> Self {
        Self {
            user_id,
            wallets: vec![],
            total_value_usd: 0.0,
            updated_at: chrono::Utc::now(),
        }
    }

    pub fn add_wallet(&mut self, wallet: WalletState) {
        self.total_value_usd += wallet.total_value_usd;
        self.wallets.push(wallet);
        self.updated_at = chrono::Utc::now();
    }

    pub fn total_by_asset_type(&self) -> HashMap<AssetType, f64> {
        let mut totals: HashMap<AssetType, f64> = HashMap::new();
        for wallet in &self.wallets {
            for pos in &wallet.positions {
                *totals.entry(pos.asset_type.clone()).or_insert(0.0) += pos.amount;
            }
        }
        totals
    }

    pub fn concentration_risk(&self, threshold: f64) -> Vec<String> {
        // Returns assets that exceed concentration threshold (% of total)
        let total = self.total_value_usd;
        if total == 0.0 {
            return vec![];
        }

        let mut asset_values: HashMap<String, f64> = HashMap::new();
        for wallet in &self.wallets {
            for pos in &wallet.positions {
                *asset_values.entry(pos.asset_symbol.clone()).or_insert(0.0) += pos.amount;
            }
        }

        asset_values
            .into_iter()
            .filter(|(_, value)| *value / total > threshold)
            .map(|(symbol, _)| symbol)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_state() {
        let user_id = uuid::Uuid::new_v4();
        let mut portfolio = PortfolioState::new(user_id);

        let wallet = WalletState {
            address: "0x1234...".into(),
            chain: ChainId::Ethereum,
            positions: vec![
                Position {
                    position_id: uuid::Uuid::new_v4(),
                    chain: ChainId::Ethereum,
                    wallet_address: "0x1234...".into(),
                    asset_type: AssetType::Native,
                    asset_id: "ETH".into(),
                    asset_symbol: "ETH".into(),
                    amount: 1.5,
                    raw_amount: 1_500_000_000_000_000_000,
                    protocol: None,
                    protocol_sub_type: None,
                    opened_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                },
                Position {
                    position_id: uuid::Uuid::new_v4(),
                    chain: ChainId::Ethereum,
                    wallet_address: "0x1234...".into(),
                    asset_type: AssetType::Stablecoin,
                    asset_id: "0xa0b8...".into(),
                    asset_symbol: "USDC".into(),
                    amount: 5000.0,
                    raw_amount: 5_000_000_000,
                    protocol: None,
                    protocol_sub_type: None,
                    opened_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                },
            ],
            total_value_usd: 6500.0,
            last_synced_block: 19_000_000,
            last_synced_at: chrono::Utc::now(),
        };

        portfolio.add_wallet(wallet);
        assert_eq!(portfolio.total_value_usd, 6500.0);
        assert_eq!(portfolio.wallets.len(), 1);

        let by_type = portfolio.total_by_asset_type();
        assert_eq!(by_type.get(&AssetType::Native), Some(&1.5));
        assert_eq!(by_type.get(&AssetType::Stablecoin), Some(&5000.0));
    }
}   