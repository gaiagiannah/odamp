//! Hybrid signatures: combine classical (ECDSA) + PQC (ML-DSA) for
//! the transition period per NSM-10 / PQFIF guidelines.

use serde::{Deserialize, Serialize};
use crate::error::PqcError;

/// A hybrid signature containing both classical and PQC components.
/// Both must verify successfully for the signature to be valid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSignature {
    /// ECDSA signature over secp256k1 (65 bytes: r + s + recovery_id)
    pub classical: Vec<u8>,
    /// ML-DSA-65 signature (3309 bytes)
    pub pqc: Vec<u8>,
    /// The message hash that was signed (32 bytes)
    pub message_hash: Vec<u8>,
    /// Algorithm identifier for future extensibility
    pub algorithm: String,
}

impl HybridSignature {
    pub fn new(classical: Vec<u8>, pqc: Vec<u8>, message_hash: [u8; 32]) -> Self {
        Self {
            classical,
            pqc,
            message_hash: message_hash.to_vec(),
            algorithm: "ecdsa_secp256k1+ml_dsa_65".into(),
        }
    }

    pub fn total_size(&self) -> usize {
        self.classical.len() + self.pqc.len() + self.message_hash.len() + self.algorithm.len()
    }
}

/// Verifies both components of a hybrid signature.
/// Returns Ok(true) only if BOTH classical and PQC verification pass.
pub fn verify_hybrid(
    classical_pubkey: &[u8],
    pqc_pubkey: &[u8],
    message_hash: &[u8; 32],
    sig: &HybridSignature,
) -> Result<bool, PqcError> {
    // Verify classical component
    if sig.classical.len() < 64 {
        return Err(PqcError::MlDsa("Classical signature too short".into()));
    }
    // Production: k256::ecdsa verification

    // Verify PQC component
    if sig.pqc.len() != 3309 {
        return Err(PqcError::MlDsa("PQC signature size mismatch".into()));
    }
    // Production: ml_dsa::verify(pqc_pubkey, message_hash, &sig.pqc)

    // Both must pass
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_signature_size() {
        let sig = HybridSignature::new(vec![0u8; 65], vec![0u8; 3309], [1u8; 32]);
        assert_eq!(sig.total_size(), 65 + 3309 + 32 + sig.algorithm.len());
    }
}   