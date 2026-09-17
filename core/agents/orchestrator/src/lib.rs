//! ODAMP Agent Orchestrator
//!
//! Coordinates all AI agents: routes user intent to the appropriate
//! agent(s), enforces guardrails (spending limits, circuit breakers,
//! human-in-the-loop), and logs all actions to the immutable audit trail.
//!
//! Design principles:
//! - Every agent action must be explainable (plain-language reasoning)
//! - No agent can exceed user-defined spending limits
//! - Circuit breakers auto-disable agents after excessive losses
//! - Human confirmation required for actions >$10,000 (configurable)
//! - Kill switch: user can halt ALL agent activity instantly
//! - All actions logged with full reasoning chain

pub mod agent;
pub mod decision_gate;
pub mod guardrails;
pub mod audit;
pub mod kill_switch;
pub mod error;

pub use agent::{Agent, AgentStatus, AgentAction, AgentResponse};
pub use decision_gate::DecisionGate;
pub use guardrails::{Guardrails, CircuitBreaker};
pub use audit::AuditLogger;
pub use kill_switch::KillSwitch;
pub use error::OrchestratorError;   