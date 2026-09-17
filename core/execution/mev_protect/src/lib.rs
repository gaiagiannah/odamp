//! ODAMP MEV Protection
//!
//! Protects user transactions from Maximal Extractable Value (MEV)
//! attacks: sandwich attacks, front-running, back-running, and
//! arbitrage extraction.
//!
//! Strategies:
//! - Private transaction mempool (Flashbots Protect, MEV-Blocker)
//! - Slippage caps with hard limits
//! - Transaction simulation before broadcast
//! - Bundle submission (atomic execution)
//! - Intent-based execution (no public mempool exposure)
//! - Time-locked execution (prevents front-running)

pub mod protection;
pub mod simulation;
pub mod bundle;
pub mod error;

pub use protection::{MevProtector, ProtectionLevel, MevThreat};
pub use simulation::TxSimulator;
pub use error::MevError;   