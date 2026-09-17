//! Core agent trait and types.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// An agent's status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Paused,
    Disabled,
    CircuitBroken,
}

/// An action proposed or executed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub agent_id: uuid::Uuid,
    pub action_type: String,
    pub description: String,
    pub reasoning: String,
    pub amount_usd: f64,
    pub parameters: serde_json::Value,
    pub confidence: f64,
    pub requires_confirmation: bool,
    pub timestamp: DateTime<Utc>,
}

/// An agent's response to a query or tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub agent_id: uuid::Uuid,
    pub agent_name: String,
    pub response_type: ResponseType,
    pub message: String,
    pub reasoning: String,
    pub actions: Vec<AgentAction>,
    pub confidence: f64,
    pub data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseType {
    Recommendation,
    Alert,
    Execution,
    Report,
    Query,
    Error,
}

/// The core trait all agents implement.
#[async_trait]
pub trait Agent: Send + Sync {
    fn id(&self) -> uuid::Uuid;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn status(&self) -> &AgentStatus;

    /// Called on each tick (periodic evaluation).
    async fn on_tick(&mut self, context: &AgentContext) -> Result<Option<AgentResponse>, Box<dyn std::error::Error + Send + Sync>>;

    /// Called when the user sends a natural language query.
    async fn on_query(&mut self, query: &str, context: &AgentContext) -> Result<AgentResponse, Box<dyn std::error::Error + Send + Sync>>;

    /// Called to configure the agent's parameters.
    fn configure(&mut self, params: serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Pauses the agent.
    fn pause(&mut self);

    /// Resumes the agent.
    fn resume(&mut self);

    /// Disables the agent permanently (until re-enabled by user).
    fn disable(&mut self, reason: &str);
}

/// Context passed to agents on each tick/query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub user_id: uuid::Uuid,
    pub timestamp: DateTime<Utc>,
    pub portfolio_value_usd: f64,
    pub portfolio_positions: serde_json::Value,
    pub market_data: serde_json::Value,
    pub active_alerts: Vec<String>,
    pub recent_events: Vec<serde_json::Value>,
    pub agent_config: serde_json::Value,
    pub spending_limit_usd: f64,
    pub daily_limit_usd: f64,
    pub kill_switch_active: bool,
}

impl AgentContext {
    pub fn new(user_id: uuid::Uuid) -> Self {
        Self {
            user_id,
            timestamp: Utc::now(),
            portfolio_value_usd: 0.0,
            portfolio_positions: serde_json::json!({}),
            market_data: serde_json::json!({}),
            active_alerts: vec![],
            recent_events: vec![],
            agent_config: serde_json::json!({}),
            spending_limit_usd: 100_000.0,
            daily_limit_usd: 50_000.0,
            kill_switch_active: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_action_creation() {
        let action = AgentAction {
            agent_id: uuid::Uuid::new_v4(),
            action_type: "rebalance".into(),
            description: "Sell 5% BTC to rebalance toward target allocation".into(),
            reasoning: "BTC is at 55% of portfolio, target is 50%. Drift exceeds 5% threshold.".into(),
            amount_usd: 5_000.0,
            parameters: serde_json::json!({"symbol": "BTC", "direction": "sell"}),
            confidence: 0.92,
            requires_confirmation: false,
            timestamp: Utc::now(),
        };

        assert!(action.confidence > 0.9);
        assert!(!action.requires_confirmation);
        assert!(!action.reasoning.is_empty());
    }

    #[test]
    fn test_agent_context() {
        let ctx = AgentContext::new(uuid::Uuid::new_v4());
        assert!(!ctx.kill_switch_active);
        assert_eq!(ctx.spending_limit_usd, 100_000.0);
    }
}   