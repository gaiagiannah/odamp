//! FROST threshold signing — 2-round protocol.
//!
//! Round 1: Each participant generates nonces + signing commitments.
//! Round 2: Each participant produces a signature share.
//! Aggregate: Coordinator combines shares into a single Schnorr signature.
//!
//! The output is a standard 64-byte Schnorr signature (R || S)
//! verifiable against the group public key.

use frost_secp256k1 as frost;
use frost_secp256k1::keys::KeyPackage;
use frost_secp256k1::round1::SigningCommitments;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

use crate::dkg::{deserialize_secret_share, key_package_from_share};
use crate::key_store::decrypt_share;

#[derive(Error, Debug)]
pub enum SignError {
    #[error("Insufficient shares: need at least {needed}, got {got}")]
    InsufficientShares { needed: usize, got: usize },
    #[error("Decryption failed for share {0}")]
    DecryptionFailed(u32),
    #[error("FROST error: {0}")]
    Frost(#[from] frost::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignResult {
    pub signature: String,
    pub shares_used: Vec<u32>,
    pub message_hash: String,
}

/// Sign a message using FROST threshold signing.
///
/// # Arguments
/// * `message` - The raw bytes to sign
/// * `encrypted_shares` - At least `threshold` encrypted key shares as (share_id, encrypted_bytes)
/// * `passwords` - Password for each share, keyed by share_id
/// * `threshold` - Minimum number of shares required
///
/// # Returns
/// A 64-byte Schnorr signature (hex-encoded) verifiable against the group public key.
pub fn sign_message(
    message: &[u8],
    encrypted_shares: &[(u32, &[u8])],
    passwords: &std::collections::HashMap<u32, &str>,
    threshold: u32,
) -> Result<SignResult, SignError> {
    if encrypted_shares.len() < threshold as usize {
        return Err(SignError::InsufficientShares {
            needed: threshold as usize,
            got: encrypted_shares.len(),
        });
    }

    let mut rng = OsRng;

    // Step 1: Decrypt shares and reconstruct KeyPackages
    let mut key_packages: BTreeMap<u32, KeyPackage> = BTreeMap::new();
    for (share_id, encrypted) in encrypted_shares.iter().take(threshold as usize) {
        let password = passwords
            .get(share_id)
            .ok_or(SignError::DecryptionFailed(*share_id))?;

        let decrypted = decrypt_share(encrypted, password)
            .map_err(|_| SignError::DecryptionFailed(*share_id))?;

        let secret_share = deserialize_secret_share(&decrypted)
            .map_err(|e| SignError::DecryptionFailed(*share_id))?;

        let kp = key_package_from_share(&secret_share)
            .map_err(|e| SignError::DecryptionFailed(*share_id))?;

        key_packages.insert(*share_id, kp);
    }

    // Step 2: Round 1 — Generate nonces and commitments for each participant
    let mut nonces_map: BTreeMap<u32, frost::round1::SigningNonces> = BTreeMap::new();
    let mut commitments_map: BTreeMap<u32, SigningCommitments> = BTreeMap::new();

    for (identifier, key_package) in &key_packages {
        let (nonces, commitments) = frost::round1::commit(
            key_package.signing_share(),
            &mut rng,
        );
        nonces_map.insert(*identifier, nonces);
        commitments_map.insert(*identifier, commitments);
    }

    // Step 3: Create the SigningPackage (coordinator step)
    let signing_package = frost::SigningPackage::new(commitments_map, message);

    // Step 4: Round 2 — Each participant generates their signature share
    let mut signature_shares: BTreeMap<u32, frost::round2::SignatureShare> = BTreeMap::new();
    for (identifier, key_package) in &key_packages {
        let nonces = &nonces_map[identifier];
        let sig_share = frost::round2::sign(&signing_package, nonces, key_package)?;
        signature_shares.insert(*identifier, sig_share);
    }

    // Step 5: Aggregate signature shares into a single group signature
    // We need the pubkey_package for aggregation — reconstruct it from
    // the verifying key (in production, this would be stored at wallet creation)
    // For v0.x, we derive it from the first KeyPackage's group public key.
    let pubkey_package = key_packages.values().next().unwrap().group_public_key().clone();

    let group_signature = frost::aggregate(&signing_package, &signature_shares, &pubkey_package)?;

    // Step 6: Encode the signature
    // frost-secp256k1 Signature is (R, S) where R is a point and S is a scalar
    // For EVM compatibility, we encode as R_x (32 bytes) || S (32 bytes) = 64 bytes
    let sig_bytes = group_signature.to_bytes();

    // Compute message hash for logging
    let message_hash = keccak256(message);

    Ok(SignResult {
        signature: hex::encode(sig_bytes),
        shares_used: key_packages.keys().cloned().collect(),
        message_hash: hex::encode(message_hash),
    })
}

/// Keccak-256 hash (EVM convention).
fn keccak256(data: &[u8]) -> [u8; 32] {
    use sha3::{Keccak256, Digest};
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dkg::generate_key_shares;
    use base64::Engine;

    #[test]
    fn test_sign_and_verify() {
        // Generate 3 shares with threshold 2
        let dkg_result = generate_key_shares(3, 2, "test_password").unwrap();

        // Use shares 1 and 2 to sign
        let message = b"hello odamp";
        let mut passwords = std::collections::HashMap::new();
        passwords.insert(1u32, "test_password");
        passwords.insert(2u32, "test_password");

        let encrypted_shares: Vec<(u32, Vec<u8>)> = dkg_result
            .shares
            .iter()
            .filter(|s| s.share_id <= 2)
            .map(|s| (s.share_id, base64::engine::general_purpose::STANDARD.decode(&s.encrypted_share).unwrap()))
            .collect();

        let sign_result = sign_message(
            message,
            &encrypted_shares.iter().map(|(id, b)| (*id, b.as_slice())).collect::<Vec<_>>(),
            &passwords,
            2,
        )
        .unwrap();

        // Signature should be 64 bytes (128 hex chars)
        assert_eq!(sign_result.signature.len(), 128);
        assert_eq!(sign_result.shares_used.len(), 2);

        // Verify the signature against the group public key
        let group_pk_hex = &dkg_result.group_public_key;
        let group_pk_bytes = hex::decode(group_pk_hex).unwrap();
        let verifying_key = frost::keys::VerifyingKey::from_bytes(&group_pk_bytes).unwrap();
        let signature = frost::Signature::from_bytes(hex::decode(&sign_result.signature).unwrap().as_slice()).unwrap();

        assert!(verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_insufficient_shares() {
        let dkg_result = generate_key_shares(3, 2, "test_password").unwrap();

        // Only provide 1 share (need 2)
        let encrypted_shares: Vec<(u32, Vec<u8>)> = dkg_result
            .shares
            .iter()
            .take(1)
            .map(|s| (s.share_id, base64::engine::general_purpose::STANDARD.decode(&s.encrypted_share).unwrap()))
            .collect();

        let mut passwords = std::collections::HashMap::new();
        passwords.insert(1u32, "test_password");

        let result = sign_message(
            b"test",
            &encrypted_shares.iter().map(|(id, b)| (*id, b.as_slice())).collect::<Vec<_>>(),
            &passwords,
            2,
        );
        assert!(result.is_err());
    }
}   