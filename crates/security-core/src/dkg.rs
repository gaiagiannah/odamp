//! Distributed Key Generation (DKG) — Trusted Dealer Model.
//!
//! Uses `frost-secp256k1::keys::generate_with_dealer` to split a
//! root key into N shares with threshold T. In v0.x, one process
//! generates all shares and distributes them. In a future version,
//! this becomes a true multi-party DKG with ML-KEM-768 key exchange.

use frost_secp256k1 as frost;
use frost_secp256k1::keys::{KeyPackage, IdentifierList, SecretKeyShare};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

use crate::key_store::encrypt_share;

#[derive(Error, Debug)]
pub enum DkgError {
    #[error("Invalid threshold: threshold ({threshold}) must be <= total_shares ({total}) and >= 2")]
    InvalidThreshold { threshold: u32, total: u32 },
    #[error("FROST key generation error: {0}")]
    Frost(#[from] frost::Error),
    #[error("Encryption error: {0}")]
    Encryption(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGenerationResult {
    pub group_public_key: String,
    pub total_shares: u32,
    pub threshold: u32,
    pub shares: Vec<EncryptedShareOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedShareOutput {
    pub share_id: u32,
    pub encrypted_share: String,
}

/// Run the DKG ceremony (trusted dealer).
///
/// Generates a root FROST key, splits it into `total_shares` shares
/// with signing threshold `threshold`. Each share is encrypted with
/// AES-256-GCM using the provided password.
///
/// The group public key is the verifying key for all future signatures.
pub fn generate_key_shares(
    total_shares: u32,
    threshold: u32,
    password: &str,
) -> Result<KeyGenerationResult, DkgError> {
    if threshold < 2 || threshold > total_shares {
        return Err(DkgError::InvalidThreshold { threshold, total: total_shares });
    }

    let mut rng = OsRng;

    // Generate key with trusted dealer
    let (shares, pubkey_package) = frost::keys::generate_with_dealer(
        total_shares,
        threshold,
        IdentifierList::Default,
        &mut rng,
    )?;

    // Group public key (hex-encoded for storage/display)
    let group_public_key = hex::encode(pubkey_package.verifying_key().to_bytes());

    // Encrypt each share and prepare for output
    let encrypted_shares: Vec<EncryptedShareOutput> = shares
        .into_iter()
        .map(|(identifier, secret_share)| {
            let id: u32 = identifier;
            // Serialize the SecretKeyShare to bytes for encryption
            let share_bytes = serialize_secret_share(&secret_share)?;
            let encrypted = encrypt_share(&share_bytes, password)
                .map_err(|e| DkgError::Encryption(e.to_string()))?;
            Ok(EncryptedShareOutput {
                share_id: id,
                encrypted_share: base64::encode(encrypted),
            })
        })
        .collect::<Result<Vec<_>, DkgError>>()?;

    Ok(KeyGenerationResult {
        group_public_key,
        total_shares,
        threshold,
        shares: encrypted_shares,
    })
}

/// Serialize a SecretKeyShare to bytes.
/// Uses the frost crate's built-in serialization (via serde).
fn serialize_secret_share(share: &SecretKeyShare) -> Result<Vec<u8>, DkgError> {
    // frost-secp256k1's SecretKeyShare implements serde::Serialize
    // when the "serde" feature is enabled (it is by default in v2.x)
    let json = serde_json::to_vec(share)
        .map_err(|e| DkgError::Encryption(format!("Serialization failed: {}", e)))?;
    Ok(json)
}

/// Deserialize a SecretKeyShare from bytes.
pub fn deserialize_secret_share(bytes: &[u8]) -> Result<SecretKeyShare, DkgError> {
    let share: SecretKeyShare = serde_json::from_slice(bytes)
        .map_err(|e| DkgError::Encryption(format!("Deserialization failed: {}", e)))?;
    Ok(share)
}

/// Reconstruct a KeyPackage from a deserialized SecretKeyShare.
pub fn key_package_from_share(share: &SecretKeyShare) -> Result<KeyPackage, DkgError> {
    let kp = KeyPackage::try_from(share.clone())?;
    Ok(kp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key_shares() {
        let result = generate_key_shares(3, 2, "test_password").unwrap();
        assert_eq!(result.total_shares, 3);
        assert_eq!(result.threshold, 2);
        assert_eq!(result.shares.len(), 3);
        assert!(!result.group_public_key.is_empty());
        // Share IDs should be 1, 2, 3
        assert_eq!(result.shares[0].share_id, 1);
        assert_eq!(result.shares[1].share_id, 2);
        assert_eq!(result.shares[2].share_id, 3);
    }

    #[test]
    fn test_invalid_threshold() {
        assert!(generate_key_shares(3, 1, "pw").is_err());
        assert!(generate_key_shares(3, 4, "pw").is_err());
    }

    #[test]
    fn test_share_roundtrip() {
        let result = generate_key_shares(3, 2, "test_password").unwrap();

        // Decrypt share 1 and verify we can reconstruct a KeyPackage
        let encrypted = base64::decode(&result.shares[0].encrypted_share).unwrap();
        let decrypted = crate::key_store::decrypt_share(&encrypted, "test_password").unwrap();
        let share: SecretKeyShare = deserialize_secret_share(&decrypted).unwrap();
        let kp = key_package_from_share(&share).unwrap();

        // The KeyPackage should have a valid signing share
        assert!(kp.signing_share().is_some());
    }
}    