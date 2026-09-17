//! MEV protection strategies and threat detection.

use serde::{Deserialize, Serialize};
use crate::error::MevError;

/// Protection level for a transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProtectionLevel {
    /// Standard: slippage cap + simulation
    Standard,
    /// Enhanced: private mempool + slippage cap + simulation
    Enhanced,
    /// Maximum: private mempool + bundle + time-lock + simulation
    Maximum,
}

/// Types of MEV threats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MevThreat {
    SandwichAttack,
    FrontRunning,
    BackRunning,
    ArbitrageExtraction,
    LiquidationSniping,
    AtomicArbitrage,
}

/// MEV protection configuration for a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevProtection {
    pub level: ProtectionLevel,
    pub max_slippage_bps: f64,
    pub hard_slippage_bps: f64,
    pub use_private_mempool: bool,
    pub use_bundle: bool,
    pub time_lock_secs: u32,
    pub min_profit_bps: f64,
}

impl Default for MevProtection {
    fn default() -> Self {
        Self {
            level: ProtectionLevel::Enhanced,
            max_slippage_bps: 50.0,
            hard_slippage_bps: 200.0,
            use_private_mempool: true,
            use_bundle: false,
            time_lock_secs: 0,
            min_profit_bps: 0.0,
        }
    }
}

/// The MEV protector: evaluates transactions for MEV risk and
/// applies appropriate protection.
pub struct MevProtector {
    default_protection: MevProtection,
    private_mempool_available: bool,
}

impl MevProtector {
    pub fn new(private_mempool_available: bool) -> Self {
        Self {
            default_protection: MevProtection::default(),
            private_mempool_available,
        }
    }

    /// Evaluates a transaction for MEV risk and returns the protection plan.
    pub fn evaluate(
        &self,
        tx_value_usd: f64,
        token_volatility_1h_pct: f64,
        is_swap: bool,
        is_large_order: bool,
    ) -> Result<ProtectionPlan, MevError> {
        let mut protection = self.default_protection.clone();

        // Escalate protection for high-value or volatile transactions
        if tx_value_usd > 10_000.0 || token_volatility_1h_pct > 5.0 {
            protection.level = ProtectionLevel::Maximum;
            protection.use_bundle = true;
        }

        if is_swap && is_large_order {
            protection.max_slippage_bps = 30.0;
            protection.hard_slippage_bps = 100.0;
        }

        if !self.private_mempool_available && protection.use_private_mempool {
            // Fall back to standard if private mempool is down
            protection.use_private_mempool = false;
            if protection.level == ProtectionLevel::Maximum {
                protection.level = ProtectionLevel::Enhanced;
            }
        }

        let threats = self.assess_threats(is_swap, is_large_order, token_volatility_1h_pct);

        Ok(ProtectionPlan {
            protection,
            detected_threats: threats,
            recommended_action: if threats.iter().any(|t| matches!(t, MevThreat::SandwichAttack)) {
                RecommendedAction::UsePrivateMempool
            } else if threats.is_empty() {
                RecommendedAction::StandardBroadcast
            } else {
                RecommendedAction::UseBundle
            },
        })
    }

    fn assess_threats(&self, is_swap: bool, is_large: bool, volatility: f64) -> Vec<MevThreat> {
        let mut threats = vec![];

        if is_swap {
            threats.push(MevThreat::SandwichAttack);
            threats.push(MevThreat::FrontRunning);
        }

        if is_large {
            threats.push(MevThreat::ArbitrageExtraction);
        }

        if volatility > 10.0 {
            threats.push(MevThreat::LiquidationSniping);
        }

        threats
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionPlan {
    pub protection: MevProtection,
    pub detected_threats: Vec<MevThreat>,
    pub recommended_action: RecommendedAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendedAction {
    StandardBroadcast,
    UsePrivateMempool,
    UseBundle,
    DelayExecution,
    SplitOrder,
}

impl Default for MevProtector {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protection_escalation() {
        let protector = MevProtector::new(true);

        // Small, stable transaction
        let plan = protector.evaluate(100.0, 1.0, true, false).unwrap();
        assert!(plan.protection.level != ProtectionLevel::Maximum);

        // Large, volatile swap
        let plan = protector.evaluate(50_000.0, 8.0, true, true).unwrap();
        assert_eq!(plan.protection.level, ProtectionLevel::Maximum);
        assert!(plan.protection.use_bundle);
    }
}   