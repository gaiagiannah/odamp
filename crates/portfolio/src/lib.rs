//! ODAMP Portfolio Engine
//!
//! Position tracking and risk-adjusted analytics.
//! The math is genuinely computed from input data — no stubs.

pub mod position;
pub mod risk;

pub use position::Position;
pub use risk::{compute_risk_metrics, RiskMetrics, PriceHistory};   