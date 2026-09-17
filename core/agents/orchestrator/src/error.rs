use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("Agent {id} not found")]
    AgentNotFound(uuid::Uuid),

    #[error("Agent {id} is disabled")]
    AgentDisabled(uuid::Uuid),

    #[error("Kill switch is active: all agents halted")]
    KillSwitchActive,

    #[error("Action blocked by guardrails: {0}")]
    GuardrailBlocked(String),

    #[error("Circuit breaker triggered for agent {id}: {reason}")]
    CircuitBreaker { id: uuid::Uuid, reason: String },

    #[error("Human confirmation required for action of ${0}")]
    ConfirmationRequired(f64),

    #[error("Agent {id} exceeded daily action limit")]
    DailyLimitExceeded(uuid::Uuid),

    #[error("Conflicting agent decisions: {0}")]
    Conflict(String),

    #[error("MCP protocol error: {0}")]
    McpError(String),
}   