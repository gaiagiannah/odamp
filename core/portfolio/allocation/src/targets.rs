//! Allocation target management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::AllocationError;

/// Asset classes supported by ODAMP.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssetClass {
    Crypto,
    Stablecoin,
    TokenizedEquity,
    TokenizedBond,
    TokenizedCommodity,
    TokenizedRealEstate,
    TokenizedInfrastructure,
    DefiYield,
    Cbdc,
}

impl AssetClass {
    pub fn name(&self) -> &'static str {
        match self {
            AssetClass::Crypto => "crypto",
            AssetClass::Stablecoin => "stablecoin",
            AssetClass::TokenizedEquity => "tokenized_equity",
            AssetClass::TokenizedBond => "tokenized_bond",
            AssetClass::TokenizedCommodity => "tokenized_commodity",
            AssetClass::TokenizedRealEstate => "tokenized_real_estate",
            AssetClass::TokenizedInfrastructure => "tokenized_infrastructure",
            AssetClass::DefiYield => "defi_yield",
            AssetClass::Cbdc => "cbdc",
        }
    }
}

/// A user's target allocation profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationProfile {
    pub name: String,
    pub risk_level: RiskLevel,
    pub targets: Vec<AssetTarget>,
    pub drift_threshold_pct: f64,
    pub rebalance_frequency: RebalanceFrequency,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Conservative,
    Moderate,
    Aggressive,
    Maximum,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RebalanceFrequency {
    Continuous,     // rebalance on every drift exceedance
    Daily,
    Weekly,
    Monthly,
    Quarterly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTarget {
    pub asset_class: AssetClass,
    pub target_pct: f64,
    pub min_pct: f64,
    pub max_pct: f64,
}

impl AllocationProfile {
    pub fn conservative() -> Self {
        Self {
            name: "Conservative".into(),
            risk_level: RiskLevel::Conservative,
            targets: vec![
                AssetTarget { asset_class: AssetClass::Stablecoin, target_pct: 40.0, min_pct: 30.0, max_pct: 50.0 },
                AssetTarget { asset_class: AssetClass::TokenizedBond, target_pct: 25.0, min_pct: 15.0, max_pct: 35.0 },
                AssetTarget { asset_class: AssetClass::TokenizedCommodity, target_pct: 15.0, min_pct: 10.0, max_pct: 20.0 },
                AssetTarget { asset_class: AssetClass::Crypto, target_pct: 10.0, min_pct: 5.0, max_pct: 15.0 },
                AssetTarget { asset_class: AssetClass::DefiYield, target_pct: 10.0, min_pct: 5.0, max_pct: 15.0 },
            ],
            drift_threshold_pct: 5.0,
            rebalance_frequency: RebalanceFrequency::Monthly,
        }
    }

    pub fn moderate() -> Self {
        Self {
            name: "Moderate".into(),
            risk_level: RiskLevel::Moderate,
            targets: vec![
                AssetTarget { asset_class: AssetClass::Crypto, target_pct: 35.0, min_pct: 25.0, max_pct: 45.0 },
                AssetTarget { asset_class: AssetClass::Stablecoin, target_pct: 20.0, min_pct: 10.0, max_pct: 30.0 },
                AssetTarget { asset_class: AssetClass::TokenizedEquity, target_pct: 15.0, min_pct: 10.0, max_pct: 20.0 },
                AssetTarget { asset_class: AssetClass::TokenizedCommodity, target_pct: 15.0, min_pct: 10.0, max_pct: 20.0 },
                AssetTarget { asset_class: AssetClass::DefiYield, target_pct: 10.0, min_pct: 5.0, max_pct: 15.0 },
                AssetTarget { asset_class: AssetClass::TokenizedRealEstate, target_pct: 5.0, min_pct: 0.0, max_pct: 10.0 },
            ],
            drift_threshold_pct: 7.0,
            rebalance_frequency: RebalanceFrequency::Weekly,
        }
    }

    pub fn aggressive() -> Self {
        Self {
            name: "Aggressive".into(),
            risk_level: RiskLevel::Aggressive,
            targets: vec![
                AssetTarget { asset_class: AssetClass::Crypto, target_pct: 50.0, min_pct: 35.0, max_pct: 65.0 },
                AssetTarget { asset_class: AssetClass::DefiYield, target_pct: 20.0, min_pct: 10.0, max_pct: 30.0 },
                AssetTarget { asset_class: AssetClass::TokenizedEquity, target_pct: 15.0, min_pct: 10.0, max_pct: 20.0 },
                AssetTarget { asset_class: AssetClass::TokenizedCommodity, target_pct: 10.0, min_pct: 5.0, max_pct: 15.0 },
                AssetTarget { asset_class: AssetClass::Stablecoin, target_pct: 5.0, min_pct: 0.0, max_pct: 10.0 },
            ],
            drift_threshold_pct: 10.0,
            rebalance_frequency: RebalanceFrequency::Daily,
        }
    }

    pub fn validate(&self) -> Result<(), AllocationError> {
        let sum: f64 = self.targets.iter().map(|t| t.target_pct).sum();
        if (sum - 100.0).abs() > 0.01 {
            return Err(AllocationError::TargetsNot100(sum));
        }
        for target in &self.targets {
            if target.min_pct > target.target_pct || target.target_pct > target.max_pct {
                return Err(AllocationError::UnknownAssetClass(target.asset_class.name().into()));
            }
        }
        Ok(())
    }
}

/// Manages the user's allocation profile and current state.
pub struct AllocationManager {
    profile: AllocationProfile,
}

impl AllocationManager {
    pub fn new(profile: AllocationProfile) -> Result<Self, AllocationError> {
        profile.validate()?;
        Ok(Self { profile })
    }

    pub fn profile(&self) -> &AllocationProfile {
        &self.profile
    }

    /// Calculates drift for each asset class given current allocation.
    pub fn calculate_drift(&self, current: &HashMap<AssetClass, f64>) -> HashMap<AssetClass, f64> {
        let mut drift = HashMap::new();
        for target in &self.profile.targets {
            let actual = current.get(&target.asset_class).copied().unwrap_or(0.0);
            drift.insert(target.asset_class.clone(), actual - target.target_pct);
        }
        drift
    }

    /// Checks if rebalancing is needed.
    pub fn needs_rebalance(&self, current: &HashMap<AssetClass, f64>) -> bool {
        let drift = self.calculate_drift(current);
        drift.values().any(|d| d.abs() > self.profile.drift_threshold_pct)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiles_validate() {
        assert!(AllocationProfile::conservative().validate().is_ok());
        assert!(AllocationProfile::moderate().validate().is_ok());
        assert!(AllocationProfile::aggressive().validate().is_ok());
    }

    #[test]
    fn test_drift_detection() {
        let mgr = AllocationManager::new(AllocationProfile::moderate()).unwrap();

        let mut current = HashMap::new();
        current.insert(AssetClass::Crypto, 45.0); // 10% over target of 35%
        current.insert(AssetClass::Stablecoin, 15.0);
        current.insert(AssetClass::TokenizedEquity, 12.0);
        current.insert(AssetClass::TokenizedCommodity, 15.0);
        current.insert(AssetClass::DefiYield, 8.0);
        current.insert(AssetClass::TokenizedRealEstate, 5.0);

        assert!(mgr.needs_rebalance(&current));
    }
}   