//! Target allocation tracking and drift detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A target allocation rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationTarget {
    pub asset_class: String,    // "crypto", "tokenized_equity", "commodities", "stablecoin", "rwa"
    pub target_pct: f64,        // 0.0 - 100.0
    pub drift_threshold_pct: f64, // alert when actual deviates by this much
}

/// Current allocation state vs. targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationState {
    pub targets: Vec<AllocationTarget>,
    pub actual: HashMap<String, f64>, // asset_class → pct of total
    pub drift: HashMap<String, f64>,  // asset_class → drift from target
    pub rebalance_needed: bool,
}

impl AllocationState {
    pub fn calculate_drift(targets: &[AllocationTarget], actual: &HashMap<String, f64>) -> Self {
        let mut drift = HashMap::new();
        let mut rebalance_needed = false;

        for target in targets {
            let current = actual.get(&target.asset_class).copied().unwrap_or(0.0);
            let d = current - target.target_pct;
            drift.insert(target.asset_class.clone(), d);
            if d.abs() > target.drift_threshold_pct {
                rebalance_needed = true;
            }
        }

        Self {
            targets: targets.to_vec(),
            actual: actual.clone(),
            drift,
            rebalance_needed,
        }
    }

    /// Generates a rebalancing plan.
    pub fn rebalance_plan(&self, total_value: f64) -> Vec<RebalanceAction> {
        self.drift
            .iter()
            .filter(|(_, d)| d.abs() > 0.5) // ignore tiny drifts
            .map(|(asset_class, drift)| RebalanceAction {
                asset_class: asset_class.clone(),
                direction: if *drift > 0.0 { "sell".into() } else { "buy".into() },
                amount_usd: (drift.abs() / 100.0) * total_value,
                priority: drift.abs(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceAction {
    pub asset_class: String,
    pub direction: String,
    pub amount_usd: f64,
    pub priority: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_detection() {
        let targets = vec![
            AllocationTarget { asset_class: "crypto".into(), target_pct: 50.0, drift_threshold_pct: 5.0 },
            AllocationTarget { asset_class: "stablecoin".into(), target_pct: 30.0, drift_threshold_pct: 5.0 },
            AllocationTarget { asset_class: "rwa".into(), target_pct: 20.0, drift_threshold_pct: 5.0 },
        ];

        let mut actual = HashMap::new();
        actual.insert("crypto".into(), 60.0); // 10% over
        actual.insert("stablecoin".into(), 25.0); // 5% under
        actual.insert("rwa".into(), 15.0); // 5% under

        let state = AllocationState::calculate_drift(&targets, &actual);
        assert!(state.rebalance_needed);
        assert_eq!(state.drift.get("crypto"), Some(&10.0));
    }
}   