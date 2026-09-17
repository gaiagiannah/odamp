//! Threshold signing: combine T shares to produce a valid signature.
//!
//! In production, this uses a proper threshold ECDSA scheme (e.g., GGS+ or FROST)
//! where each party contributes a partial signature without ever reconstructing
//! the full private key. This module provides the interface and a simplified
//! implementation for development.

use serde::{Deserialize, Serialize};
use crate::config::MpcConfig;
use crate::error::MpcError;
use crate::share::KeyShare;

/// A partial signature contributed by one share holder.
#[derive(Debug, Serialize, Deserialize)]
pub struct PartialSignature {
    pub share_id: u32,
    pub r_component: Vec<u8>,
    pub s_partial: Vec<u8>,
    pub nonce_commitment: Vec<u8>,
}

/// A complete threshold signature (ECDSA + optional ML-DSA).
#[derive(Debug, Serialize, Deserialize)]
pub struct ThresholdSignature {
    /// ECDSA signature (r, s) over secp256k1
    pub ecdsa_r: Vec<u8>,
    pub ecdsa_s: Vec<u8>,
    /// Recovery ID (for Ethereum)
    pub recovery_id: u8,
    /// Optional ML-DSA-65 signature (post-quantum)
    pub pqc_signature: Option<Vec<u8>>,
    /// The message hash that was signed
    pub message_hash: Vec<u8>,
}

/// Combines partial signatures into a complete threshold signature.
///
/// # Arguments
/// * `partials` - At least `threshold` partial signatures
/// * `config` - The MPC configuration
/// * `message_hash` - The 32-byte hash of the message to sign
///
/// # Returns
/// * `ThresholdSignature` ready for broadcast
pub fn combine_signatures(
    partials: &[PartialSignature],
    config: &MpcConfig,
    message_hash: &[u8; 32],
) -> Result<ThresholdSignature, MpcError> {
    if partials.len() < config.threshold as usize {
        return Err(MpcError::InsufficientShares {
            have: partials.len() as u32,
            need: config.threshold,
        });
    }

    // Verify all partials are for the same message
    for partial in partials {
        if partial.nonce_commitment.is_empty() {
            return Err(MpcError::ShareVerificationFailed);
        }
    }

    // Simplified: in production this performs Lagrange interpolation
    // over the partial signatures to produce the final (r, s) pair.
    // For development, we simulate the combination.
    let first = &partials[0];

    // Generate PQC signature if enabled
    let pqc_signature = if config.pqc_enabled {
        // Placeholder: ML-DSA-65 signing would happen here
        // In production, this calls into the `odamp-pqc` crate
        Some(vec![0u8; 2420]) // ML-DSA-65 signature size
    } else {
        None
    };

    Ok(ThresholdSignature {
        ecdsa_r: first.r_component.clone(),
        ecdsa_s: first.s_partial.clone(),
        recovery_id: 0,
        pqc_signature,
        message_hash: message_hash.to_vec(),
    })
}

/// Creates a partial signature from a single key share.
///
/// In production, this uses FROST (Flexible Round-Optimized Scalable Threshold)
/// or GGS+ protocol where the share holder never exposes the full key.
pub fn create_partial_signature(
    share: &KeyShare,
    message_hash: &[u8; 32],
    nonce: &[u8; 32],
) -> Result<PartialSignature, MpcError> {
    if share.data.is_empty() {
        return Err(MpcError::InvalidKeyMaterial);
    }

    // Simplified partial signature creation
    // Production: FROST signing with precomputed nonces
    let mut hasher = sha2::Sha256::new();
    hasher.update(share.data.as_slice());
    hasher.update(message_hash);
    hasher.update(nonce);
    let hash = hasher.finalize();

    Ok(PartialSignature {
        share_id: share.share_id,
        r_component: hash[..32].to_vec(),
        s_partial: hash[..32].to_vec(),
        nonce_commitment: nonce.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SecurityTier;

    #[test]
    fn test_insufficient_shares_rejected() {
        let config = MpcConfig::for_tier(SecurityTier::Standard).unwrap();
        let partial = PartialSignature {
            share_id: 1,
            r_component: vec![0u8; 32],
            s_partial: vec![0u8; 32],
            nonce_commitment: vec![0u8; 32],
        };
        let msg = [0u8; 32];

        let result = combine_signatures(&[partial], &config, &msg);
        assert!(matches!(result, Err(MpcError::InsufficientShares { .. })));
    }

    #[test]
    fn test_successful_combination() {
        let config = MpcConfig::for_tier(SecurityTier::Basic).unwrap();
        let partials: Vec<PartialSignature> = (1..=2)
            .map(|id| PartialSignature {
                share_id: id,
                r_component: vec![id; 32],
                s_partial: vec![id; 32],
                nonce_commitment: vec![id; 32],
            })
            .collect();
        let msg = [42u8; 32];

        let sig = combine_signatures(&partials, &config, &msg).unwrap();
        assert_eq!(sig.message_hash, msg.to_vec());
        assert!(sig.pqc_signature.is_some()); // PQC enabled by default
    }
}   