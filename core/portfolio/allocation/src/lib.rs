//! Allocation engine: target management, drift detection, rebalancing logic.

pub mod targets;
pub mod rebalancer;
pub mod risk_overlay;
pub mod error;

pub use targets::AllocationManager;
pub use rebalancer::Rebalancer;
pub use error::AllocationError;   