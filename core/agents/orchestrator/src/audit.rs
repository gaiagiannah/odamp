//! Immutable audit log for all agent actions.
//! Every action is logged with full reasoning chain.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A single audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub entry_id: u64,
    pub timestamp: DateTime<Utc>,
    pub agent_id: uuid::Uuid,
    pub agent_name: String,
    pub action_type: String,
    pub description: String,
    pub reasoning: String,
    pub amount_usd: f64,
    pub gate_decision: String,
    pub gate_notes: Vec<String>,
    pub result: String,
    pub parameters: serde_json::Value,
}

/// The audit logger.
pub struct AuditLogger {
    entries: Vec<AuditEntry>,
    next_id: u64,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self { entries: vec![], next_id: 1 }
    }

    pub fn log(&mut self, entry: AuditEntry) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(entry);
        id
    }

    /// Queries audit log for a specific agent.
    pub fn for_agent(&self, agent_id: &uuid::Uuid) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| &e.agent_id == agent_id).collect()
    }

    /// Queries audit log for a time range.
    pub fn between(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .collect()
    }

    /// Returns total actions by type.
    pub fn summary(&self) -> std::collections::HashMap<String, u32> {
        let mut counts = std::collections::HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.action_type.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logging() {
        let mut logger = AuditLogger::new();

        logger.log(AuditEntry {
            entry_id: 0,
            timestamp: Utc::now(),
            agent_id: uuid::Uuid::new_v4(),
            agent_name: "PortfolioManager".into(),
            action_type: "rebalance".into(),
            description: "Rebalanced BTC from 55% to 50%".into(),
            reasoning: "Drift exceeded 5% threshold".into(),
            amount_usd: 5_000.0,
            gate_decision: "Approved".into(),
            gate_notes: vec![],
            result: "success".into(),
            parameters: serde_json::json!({"symbol": "BTC"}),
        });

        assert_eq!(logger.len(), 1);
        assert!(!logger.is_empty());

        let summary = logger.summary();
        assert_eq!(summary.get("rebalance"), Some(&1));
    }
}    