//! ODAMP Key Recovery Module
//!
//! Provides multiple recovery mechanisms:
//! - Shamir's Secret Sharing (mathematical, no trusted party)
//! - Social recovery (trusted contacts with threshold)
//! - Time-locked recovery (emergency access after N days)
//!
//! All recovery operations are logged to the immutable audit trail.

pub mod sss;
pub mod social;
pub mod time_lock;
pub mod error;

pub use error::RecoveryError;   