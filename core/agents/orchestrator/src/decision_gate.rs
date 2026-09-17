//! Decision Gate: the final checkpoint before any agent action is executed.
//! All agents must pass through the gate. The gate enforces:
//! - Spending limits
//! - Human confirmation thresholds
//! - Kill switch
//! - Circuit breaker status
//! - Conflict detection (multiple agents proposing conflicting actions)

use serde::{Deserialize, Serialize};
use crate::agent::AgentAction;
use crate::guardrails::Guardrails;
use crate::error::OrchestratorError;

/// The result of passing an action through the decision gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateDecision {
    /// Action approved for execution.
    Approved { action: AgentAction, gate_notes: Vec<String> },
    /// Action requires human confirmation.
    NeedsConfirmation { action: AgentAction, reason: String },
    /// Action blocked by guardrails.
    Blocked { action: AgentAction, reason: String },
    /// Action delayed (cooling-off period).
    Delayed { action: AgentAction, delay_secs: u64 },
}

/// The decision gate.
pub struct DecisionGate {
    guardrails: Guardrails,
    confirmation_threshold_usd: f64,
    cooling_off_secs: u64,
}

impl DecisionGate {
    pub fn new(guardrails: Guardrails, confirmation_threshold: f64) -> Self {
        Self {
            guardrails,
            confirmation_threshold_usd: confirmation_threshold,
            cooling_off_secs: 30,
        }
    }

    /// Evaluates an agent action against all guardrails.
    pub fn evaluate(&self, action: &AgentAction, kill_switch_active: bool) -> Result<GateDecision, OrchestratorError> {
        // 1. Kill switch check
        if kill_switch_active {
            return Ok(GateDecision::Blocked {
                action: action.clone(),
                reason: "Kill switch is active. All agent actions are halted.".into(),
            });
        }

        // 2. Confidence check
        if action.confidence < 0.5 {
            return Ok(GateDecision::Blocked {
                action: action.clone(),
                reason: format!(
                    "Agent confidence {:.2} below minimum 0.50. Action: {}",
                    action.confidence, action.description
                ),
            });
        }

        // 3. Spending limit check
        if let Some(reason) = self.guardrails.check_spending(action.amount_usd) {
            return Ok(GateDecision::Blocked {
                action: action.clone(),
                reason,
            });
        }

        // 4. Human confirmation threshold
        if action.amount_usd > self.confirmation_threshold_usd {
            return Ok(GateDecision::NeedsConfirmation {
                action: action.clone(),
                reason: format!(
                    "Action of ${:.0} exceeds confirmation threshold ${:.0}. \
                     Reasoning: {}",
                    action.amount_usd, self.confirmation_threshold_usd, action.reasoning
                ),
            });
        }

        // 5. Circuit breaker check
        if let Some(reason) = self.guardrails.check_circuit_breaker(&action.agent_id) {
            return Ok(GateDecision::Blocked {
                action: action.clone(),
                reason,
            });
        }

        // 6. All checks passed
        let mut notes = vec![];
        if action.confidence < 0.8 {
            notes.push(format!("Low confidence ({:.2}) — monitor closely", action.confidence));
        }
        if action.amount_usd > self.confirmation_threshold_usd * 0.5 {
            notes.push("Large action — logging for review".to_string());
        }

        Ok(GateDecision::Approved {
            action: action.clone(),
            gate_notes: notes,
        })
    }

    /// Checks for conflicts between multiple agent actions.
    pub fn detect_conflicts(actions: &[AgentAction]) -> Vec<String> {
        let mut conflicts = vec![];

        for i in 0..actions.len() {
            for j in (i + 1)..actions.len() {
                let a = &actions[i];
                let b = &actions[j];

                // Same asset, opposite directions
                if a.action_type == b.action_type && a.parameters != b.parameters {
                    let a_dir = a.parameters.get("direction").and_then(|v| v.as_str()).unwrap_or("");
                    let b_dir = b.parameters.get("direction").and_then(|v| v.as_str()).unwrap_or("");
                    let a_asset = a.parameters.get("symbol").and_then(|v| v.as_str()).unwrap_or("");
                    let b_asset = b.parameters.get("symbol").and_then(|v| v.as_str()).unwrap_or("");

                    if a_asset == b_asset && a_dir != b_dir {
                        conflicts.push(format!(
                            "Conflict: Agent {} wants to {} {}, Agent {} wants to {} {}",
                            a.agent_id, a_dir, a_asset,
                            b.agent_id, b_dir, b_asset
                        ));
                    }
                }
            }
        }

        conflicts
    }
}

impl Default for DecisionGate {
    fn default() -> Self {
        Self::new(Guardrails::default(), 10_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentAction;

    fn make_action(amount: f64, confidence: f64) -> AgentAction {
        AgentAction {
            agent_id: uuid::Uuid::new_v4(),
            action_type: "trade".into(),
            description: "Test action".into(),
            reasoning: "Test reasoning".into(),
            amount_usd: amount,
            parameters: serde_json::json!({}),
            confidence,
            requires_confirmation: false,
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_gate_approves_small_confident_action() {
        let gate = DecisionGate::default();
        let action = make_action(500.0, 0.95);

        let decision = gate.evaluate(&action, false).unwrap();
        assert!(matches!(decision, GateDecision::Approved { .. }));
    }

    #[test]
    fn test_gate_blocks_kill_switch() {
        let gate = DecisionGate::default();
        let action = make_action(500.0, 0.95);

        let decision = gate.evaluate(&action, true).unwrap();
        assert!(matches!(decision, GateDecision::Blocked { .. }));
    }

    #[test]
    fn test_gate_requires_confirmation_for_large() {
        let gate = DecisionGate::default(); // threshold $10K
        let action = make_action(25_000.0, 0.95);

        let decision = gate.evaluate(&action, false).unwrap();
        assert!(matches!(decision, GateDecision::NeedsConfirmation { .. }));
    }

    #[test]
    fn test_gate_blocks_low_confidence() {
        let gate = DecisionGate::default();
        let action = make_action(500.0, 0.3);

        let decision = gate.evaluate(&action, false).unwrap();
        assert!(matches!(decision, GateDecision::Blocked { .. }));
    }

    #[test]
    fn test_conflict_detection() {
        let buy = AgentAction {
            agent_id: uuid::Uuid::new_v4(),
            action_type: "trade".into(),
            description: "Buy BTC".into(),
            reasoning: "Momentum".into(),
            amount_usd: 1000.0,
            parameters: serde_json::json!({"symbol": "BTC", "direction": "buy"}),
            confidence: 0.9,
            requires_confirmation: false,
            timestamp: chrono::Utc::now(),
        };

        let sell = AgentAction {
            agent_id: uuid::Uuid::new_v4(),
            action_type: "trade".into(),
            description: "Sell BTC".into(),
            reasoning: "Overallocated".into(),
            amount_usd: 1000.0,
            parameters: serde_json::json!({"symbol": "BTC", "direction": "sell"}),
            confidence: 0.9,
            requires_confirmation: false,
            timestamp: chrono::Utc::now(),
        };

        let conflicts = DecisionGate::detect_conflicts(&[buy, sell]);
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("Conflict"));
    }
}   