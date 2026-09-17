//! ODAMP Zero-Knowledge Proofs Module
//!
//! Provides ZK proof generation and verification for:
//! - Balance verification (prove you have >= X without revealing exact amount)
//! - Range proofs (prove value is within [min, max] without revealing value)
//! - Selective disclosure (reveal specific transactions to auditor, hide rest)
//!
//! Production will use Halo2 or Groth16 (via arkworks). This module
//! defines the interface and provides placeholder implementations.

pub mod balance_proof;
pub mod range_proof;
pub mod selective_disclosure;
pub mod error;

pub use error::ZkError;   