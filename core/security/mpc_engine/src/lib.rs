//! ODAMP MPC Engine
//!
//! Multi-Party Computation threshold signature engine with post-quantum
//! hybrid signatures. Supports (2-of-3) and (3-of-5) configurations.
//!
//! Security model:
//! - No single party (including ODAMP operators) holds the full private key
//! - Key shares are distributed across user devices and optional HSMs
//! - Signatures require threshold cooperation
//! - Hybrid ECDSA + ML-DSA for post-quantum security

pub mod keygen;
pub mod sign;
pub mod verify;
pub mod share;
pub mod config;
pub mod error;
pub mod recovery;

pub use config::MpcConfig;
pub use error::MpcError;
pub use keygen::KeyGenResult;
pub mod types;   