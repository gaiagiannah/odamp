//! DeFi-specific tax event classification.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Types of DeFi events that have tax implications.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DefiEventType {
    /// Staking rewards → ordinary income at receipt
    StakingReward,
    /// LP fees earned → ordinary income
    LpFees,
    /// Airdrop received → ordinary income at fair market value
    Airdrop,
    /// Governance token distribution → ordinary income
    GovernanceToken,
    /// Yield from lending → interest income
    LendingYield,
    /// NFT sale → capital gain
    NftSale,
    /// Protocol migration (e.g., AAVE → AAVEv3) → non-taxable (same asset)
    ProtocolMigration,
    /// Token swap (e.g., USDC → DAI) → taxable event (disposal + acquisition)
    TokenSwap,
    /// Liquidation → capital gain/loss
    Liquidation,
    /// Burn → non-taxable
    Burn,
    /// Airdrop of previously staked tokens → potentially non-taxable
    StakedAirdrop,
}

impl DefiEventType {
    pub fn is_taxable(&self) -> bool {
        !matches!(self, DefiEventType::ProtocolMigration | DefiEventType::Burn)
    }

    pub fn income_type(&self) -> Option<IncomeType> {
        match self {
            DefiEventType::StakingReward
            | DefiEventType::LpFees
            | DefiEventType::Airdrop
            | DefiEventType::GovernanceToken => Some(IncomeType::Ordinary),
            DefiEventType::LendingYield => Some(IncomeType::Interest),
            DefiEventType::NftSale
            | DefiEventType::TokenSwap
            | DefiEventType::Liquidation => Some(IncomeType::CapitalGain),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IncomeType {
    Ordinary,
    Interest,
    CapitalGain,
}

/// A classified DeFi tax event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefiTaxEvent {
    pub event_type: DefiEventType,
    pub asset_symbol: String,
    pub amount: f64,
    pub value_usd: f64,
    pub protocol: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub tx_hash: Option<String>,
    pub is_taxable: bool,
    pub income_type: Option<IncomeType>,
}

impl DefiTaxEvent {
    pub fn new(
        event_type: DefiEventType,
        symbol: &str,
        amount: f64,
        value_usd: f64,
        protocol: Option<&str>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            is_taxable: event_type.is_taxable(),
            income_type: event_type.income_type(),
            event_type,
            asset_symbol: symbol.to_string(),
            amount,
            value_usd,
            protocol: protocol.map(|s| s.to_string()),
            timestamp,
            tx_hash: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_classification() {
        assert!(DefiEventType::StakingReward.is_taxable());
        assert!(!DefiEventType::ProtocolMigration.is_taxable());
        assert_eq!(DefiEventType::StakingReward.income_type(), Some(IncomeType::Ordinary));
        assert_eq!(DefiEventType::NftSale.income_type(), Some(IncomeType::CapitalGain));
        assert_eq!(DefiEventType::LendingYield.income_type(), Some(IncomeType::Interest));
    }
}   