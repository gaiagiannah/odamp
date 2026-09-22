//! ODAMP Security Core
//!
//! FROST 2-of-3 threshold signing (secp256k1) with AES-256-GCM key storage.
//! The full private key NEVER exists in any single process.

pub mod dkg;
pub mod sign;
pub mod key_store;

pub use dkg::{generate_key_shares, KeyGenerationResult, EncryptedShareOutput};
pub use sign::{sign_message, SignResult};
pub use key_store::{KeyStore, EncryptedShareFile};   