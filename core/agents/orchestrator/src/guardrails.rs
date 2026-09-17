//! Guardrails: spending limits, circuit breakers, rate limiting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Guardrail configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guardrails {
    pub max_single_action_usd: f64,
    pub max_daily_spend_usd: f64,
    pub max_weekly_spend_usd: f64,
    pub max_actions_per_hour: u32,
    pub circuit_breaker_loss_pct: f64,
    pub circuit_breaker_cooldown_secs: u64,
}

impl Default for Guardrails {
    fn default() -> Self {
        Self {
            max_single_action_usd: 50_000.0,
            max_daily_spend_usd: 100_000.0,
            max_weekly_spend_usd: 250_000.0,
            max_actions_per_hour: 20,
            circuit_breaker_loss_pct: 5.0,
            circuit_breaker_cooldown_secs: 3600,
        }
    }
}

/// Tracks spending and triggers circuit breakers.
pub struct CircuitBreaker {
    pub agent_id: uuid::Uuid,
    pub triggered_at: Option<DateTime<Utc>>,
    pub trigger_reason: String,
    pub loss_pct: f64,
    pub cooldown_secs: u64,
}

impl CircuitBreaker {
    pub fn new(agent_id: uuid::Uuid, cooldown_secs: u64) -> Self {
        Self {
            agent_id,
            triggered_at: None,
            trigger_reason: String::new(),
            loss_pct: 0.0,
            cooldown_secs,
        }
    }

    pub fn is_triggered(&self) -> bool {
        self.triggered_at.is_some()
    }

    pub fn is_cooldown_expired(&self) -> bool {
        match self.triggered_at {
            None => false,
            Some(t) => (Utc::now() - t).num_seconds() as u64 >= self.cooldown_secs,
        }
    }

    pub fn trigger(&mut self, reason: &str, loss_pct: f64) {
        self.triggered_at = Some(Utc::now());
        self.trigger_reason = reason.to_string();
        self.loss_pct = loss_pct;
    }

    pub fn reset(&mut self) {
        self.triggered_at = None;
        self.trigger_reason.clear();
        self.loss_pct = 0.0;
    }
}

impl Guardrails {
    /// Checks if an action exceeds spending limits.
    pub fn check_spending(&self, amount_usd: f64) -> Option<String> {
        if amount_usd > self.max_single_action_usd {
            return Some(format!(
                "Action ${:.0} exceeds max single action limit ${:.0}",
                amount_usd, self.max_single_action_usd
            ));
        }
        None
    }

    /// Checks if a circuit breaker is active for an agent.
    pub fn check_circuit_breaker(&self, _agent_id: &uuid::Uuid) -> Option<String> {
        // In production: look up circuit breaker state from shared store
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_trigger() {
        let mut cb = CircuitBreaker::new(uuid::Uuid::new_v4(), 3600);
        assert!(!cb.is_triggered());

        cb.trigger("Loss exceeded 5% threshold", 5.2);
        assert!(cb.is_triggered());
        assert!(!cb.is_cooldown_expired());
    }

    #[test]
    fn test_spending_check() {
        let guardrails = Guardrails::default();
        assert!(guardrails.check_spending(10_000.0).is_none());
        assert!(guardrails.check_spending(60_000.0).is_some());
    }
}   