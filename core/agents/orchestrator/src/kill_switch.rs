//! Kill Switch: instantly halts ALL agent activity.
//!
//! Can be triggered by:
//! - User (manual, via API or UI)
//! - System (automatic, on critical security event)
//! - Agent (self-reported critical error)
//!
//! When active:
//! - All agent ticks are ignored
//! - All pending actions are cancelled
//! - No new actions can be proposed
//! - User must manually re-enable each agent

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use parking_lot::RwLock;

/// Kill switch state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillSwitchState {
    pub active: bool,
    pub triggered_at: Option<DateTime<Utc>>,
    pub triggered_by: KillSwitchTrigger,
    pub reason: String,
    pub deactivated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KillSwitchTrigger {
    UserManual,
    SystemSecurity,
    AgentError { agent_id: uuid::Uuid },
    LossThreshold,
    ExternalEvent,
}

/// The kill switch. Thread-safe for use across async tasks.
#[derive(Clone)]
pub struct KillSwitch {
    state: Arc<RwLock<KillSwitchState>>,
}

impl KillSwitch {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(KillSwitchState {
                active: false,
                triggered_at: None,
                triggered_by: KillSwitchTrigger::UserManual,
                reason: String::new(),
                deactivated_at: None,
            })),
        }
    }

    /// Activates the kill switch. All agents halt immediately.
    pub fn activate(&self, trigger: KillSwitchTrigger, reason: &str) {
        let mut state = self.state.write();
        state.active = true;
        state.triggered_at = Some(Utc::now());
        state.triggered_by = trigger;
        state.reason = reason.to_string();
        state.deactivated_at = None;

        tracing::error!(trigger = ?trigger, reason, "KILL SWITCH ACTIVATED — all agents halted");
    }

    /// Deactivates the kill switch. Agents must be individually re-enabled.
    pub fn deactivate(&self) {
        let mut state = self.state.write();
        state.active = false;
        state.deactivated_at = Some(Utc::now());
        tracing::info!("Kill switch deactivated. Agents require manual re-enable.");
    }

    /// Checks if the kill switch is currently active.
    pub fn is_active(&self) -> bool {
        self.state.read().active
    }

    /// Gets the current state.
    pub fn state(&self) -> KillSwitchState {
        self.state.read().clone()
    }
}

impl Default for KillSwitch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kill_switch_lifecycle() {
        let ks = KillSwitch::new();
        assert!(!ks.is_active());

        ks.activate(KillSwitchTrigger::UserManual, "User pressed emergency stop");
        assert!(ks.is_active());
        assert_eq!(ks.state().triggered_by, KillSwitchTrigger::UserManual);

        ks.deactivate();
        assert!(!ks.is_active());
        assert!(ks.state().deactivated_at.is_some());
    }

    #[test]
    fn test_kill_switch_security_trigger() {
        let ks = KillSwitch::new();

        ks.activate(
            KillSwitchTrigger::SystemSecurity,
            "Critical vulnerability detected in agent communication channel",
        );

        assert!(ks.is_active());
        assert!(ks.state().reason.contains("vulnerability"));
    }
}   