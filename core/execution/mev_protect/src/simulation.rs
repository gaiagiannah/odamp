//! Transaction simulation: simulate before broadcasting to detect
//! reverts, unexpected state changes, and MEV extraction.

use serde::{Deserialize, Serialize};
use crate::error::MevError;

/// Result of a transaction simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub success: bool,
    pub gas_used: u64,
    pub gas_cost_usd: f64,
    pub state_changes: Vec<StateChange>,
    pub logs: Vec<String>,
    pub error: Option<String>,
    pub estimated_mev_loss_usd: f64,
    pub is_safe: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub address: String,
    pub key: String,
    pub old_value: String,
    pub new_value: String,
    pub is_risky: bool,
}

/// Simulates transactions before execution.
pub struct TxSimulator {
    max_gas_cost_usd: f64,
    max_mev_loss_usd: f64,
}

impl TxSimulator {
    pub fn new(max_gas_cost: f64, max_mev_loss: f64) -> Self {
        Self {
            max_gas_cost_usd: max_gas_cost,
            max_mev_loss_usd: max_mev_loss,
        }
    }

    /// Simulates a transaction and checks for issues.
    pub fn simulate(&self, tx_data: &TxToSimulate) -> Result<SimulationResult, MevError> {
        // Production: send to Tenderly, Anvil, or local EVM
        // For development: return simulated result

        let gas_cost = 3.5; // estimated
        let mut warnings = vec![];
        let mut is_safe = true;

        if gas_cost > self.max_gas_cost_usd {
            warnings.push(format!(
                "Gas cost ${:.2} exceeds limit ${:.2}",
                gas_cost, self.max_gas_cost_usd
            ));
            is_safe = false;
        }

        let estimated_mev = if tx_data.is_swap {
            tx_data.value_usd * 0.001 // 0.1% estimated MEV
        } else {
            0.0
        };

        if estimated_mev > self.max_mev_loss_usd {
            warnings.push(format!(
                "Estimated MEV loss ${:.2} exceeds limit ${:.2}",
                estimated_mev, self.max_mev_loss_usd
            ));
            is_safe = false;
        }

        // Check for dangerous state changes (unlimited approval, etc.)
        if tx_data.data.contains(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]) {
            warnings.push("UNLIMITED APPROVAL DETECTED".to_string());
            is_safe = false;
        }

        Ok(SimulationResult {
            success: true,
            gas_used: 21_000,
            gas_cost_usd: gas_cost,
            state_changes: vec![],
            logs: vec!["Simulation complete".into()],
            error: None,
            estimated_mev_loss_usd: estimated_mev,
            is_safe,
            warnings,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxToSimulate {
    pub to: String,
    pub from: String,
    pub value_usd: f64,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub is_swap: bool,
}

impl Default for TxSimulator {
    fn default() -> Self {
        Self::new(50.0, 10.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_safe_tx() {
        let simulator = TxSimulator::default();
        let tx = TxToSimulate {
            to: "0x1234".into(),
            from: "0x5678".into(),
            value_usd: 100.0,
            data: vec![0xa9, 0x05, 0x9c], // transfer
            gas_limit: 100_000,
            is_swap: false,
        };

        let result = simulator.simulate(&tx).unwrap();
        assert!(result.success);
        assert!(result.is_safe);
    }

    #[test]
    fn test_simulation_detects_unlimited_approval() {
        let simulator = TxSimulator::default();
        let tx = TxToSimulate {
            to: "0x1234".into(),
            from: "0x5678".into(),
            value_usd: 0.0,
            data: vec![
                0x09, 0x5e, 0x7b, 0x3e, // approve selector
                0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, // token
                0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, // spender
                0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
                0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
                0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff, // max uint256
            ],
            gas_limit: 100_000,
            is_swap: false,
        };

        let result = simulator.simulate(&tx).unwrap();
        assert!(!result.is_safe);
        assert!(result.warnings.iter().any(|w| w.contains("UNLIMITED")));
    }
}   