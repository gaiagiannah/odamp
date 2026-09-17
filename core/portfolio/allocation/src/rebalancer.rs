//! Rebalancing logic: generates and executes rebalance orders.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::targets::{AllocationManager, AssetClass};
use crate::error::AllocationError;

/// A single rebalance order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceOrder {
    pub asset_class: AssetClass,
    pub direction: Direction,
    pub amount_usd: f64,
    pub priority: f64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    Buy,
    Sell,
}

/// A complete rebalancing plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalancePlan {
    pub orders: Vec<RebalanceOrder>,
    pub total_buy_usd: f64,
    pub total_sell_usd: f64,
    pub net_usd: f64,
    pub estimated_slippage_bps: f64,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Generates rebalancing plans from drift analysis.
pub struct Rebalancer {
    manager: AllocationManager,
    max_single_order_usd: f64,
    max_daily_spend_usd: f64,
}

impl Rebalancer {
    pub fn new(manager: AllocationManager, max_single_order: f64, max_daily_spend: f64) -> Self {
        Self {
            manager,
            max_single_order_usd: max_single_order,
            max_daily_spend_usd: max_daily_spend,
        }
    }

    /// Generates a rebalancing plan given current allocation and total portfolio value.
    pub fn generate_plan(
        &self,
        current: &HashMap<AssetClass, f64>,
        total_value_usd: f64,
    ) -> Result<RebalancePlan, AllocationError> {
        let drift = self.manager.calculate_drift(current);
        let profile = self.manager.profile();

        let mut orders = Vec::new();
        let mut total_buy = 0.0;
        let mut total_sell = 0.0;

        for (asset_class, drift_pct) in &drift {
            if drift_pct.abs() < profile.drift_threshold_pct {
                continue;
            }

            let amount = (drift_pct.abs() / 100.0) * total_value_usd;
            let capped_amount = amount.min(self.max_single_order_usd);

            let order = RebalanceOrder {
                asset_class: asset_class.clone(),
                direction: if *drift_pct > 0.0 { Direction::Sell } else { Direction::Buy },
                amount_usd: capped_amount,
                priority: drift_pct.abs(),
                reason: format!(
                    "{} is {}% {} target (threshold: {}%)",
                    asset_class.name(),
                    drift_pct.abs(),
                    if *drift_pct > 0.0 { "above" } else { "below" },
                    profile.drift_threshold_pct
                ),
            };

            if order.direction == Direction::Buy {
                total_buy += order.amount_usd;
            } else {
                total_sell += order.amount_usd;
            }

            orders.push(order);
        }

        // Sort by priority (largest drift first)
        orders.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

        // Check daily spend limit
        if total_buy > self.max_daily_spend_usd {
            // Scale down proportionally
            let scale = self.max_daily_spend_usd / total_buy;
            for order in &mut orders {
                if order.direction == Direction::Buy {
                    order.amount_usd *= scale;
                }
            }
            total_buy = self.max_daily_spend_usd;
        }

        Ok(RebalancePlan {
            total_buy_usd: total_buy,
            total_sell_usd: total_sell,
            net_usd: total_buy - total_sell,
            estimated_slippage_bps: 5.0, // placeholder
            orders,
            generated_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::targets::AllocationProfile;

    #[test]
    fn test_rebalance_plan_generation() {
        let mgr = AllocationManager::new(AllocationProfile::moderate()).unwrap();
        let rebalancer = Rebalancer::new(mgr, 50_000.0, 100_000.0);

        let mut current = HashMap::new();
        current.insert(AssetClass::Crypto, 50.0); // 15% over
        current.insert(AssetClass::Stablecoin, 10.0); // 10% under
        current.insert(AssetClass::TokenizedEquity, 15.0);
        current.insert(AssetClass::TokenizedCommodity, 15.0);
        current.insert(AssetClass::DefiYield, 8.0);
        current.insert(AssetClass::TokenizedRealEstate, 2.0);

        let plan = rebalancer.generate_plan(&current, 100_000.0).unwrap();
        assert!(!plan.orders.is_empty());
        assert!(plan.total_sell_usd > 0.0);
        assert!(plan.total_buy_usd > 0.0);

        // First order should be the largest drift (crypto sell)
        assert_eq!(plan.orders[0].asset_class, AssetClass::Crypto);
        assert_eq!(plan.orders[0].direction, Direction::Sell);
    }
}   