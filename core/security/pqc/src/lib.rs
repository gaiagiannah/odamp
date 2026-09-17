//! ODAMP Post-Quantum Cryptography Module
//!
//! Implements NIST-standardized PQC algorithms:
//! - ML-DSA (FIPS 204) — Module-Lattice-based Digital Signature Algorithm
//! - SLH-DSA (FIPS 204) — Stateless Hash-based Digital Signature Algorithm
//! - ML-KEM (FIPS 205) — Module-Lattice-based Key Encapsulation Mechanism
//!
//! All operations produce hybrid signatures (classical + PQC) during
//! the transition period, per NSM-10 / PQFIF guidelines.

pub mod ml_dsa;
pub mod slh_dsa;
pub mod ml_kem;
pub mod hybrid;
pub mod error;

pub use error::PqcError;
pub use hybrid::HybridSignature;   