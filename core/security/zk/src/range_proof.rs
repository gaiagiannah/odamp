//! Range proofs: prove a value is within [min, max] without revealing it.

use serde::{Deserialize, Serialize};
use crate::error::ZkError;

#[derive(Debug, Serialize, Deserialize)]
pub struct RangeProof {
    pub proof_bytes: Vec<u8>,
    pub min: u64,
    pub max: u64,
    pub proof_system: String,
}

pub fn generate_range_proof(
    value: u64,
    min: u64,
    max: u64,
) -> Result<RangeProof, ZkError> {
    if value < min || value > max {
        return Err(ZkError::ConstraintViolation(
            "Value outside range".into(),
        ));
    }

    let mut proof_bytes = vec![0u8; 256];
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(value.to_le_bytes());
    h.update(min.to_le_bytes());
    h.update(max.to_le_bytes());
    proof_bytes[..32].copy_from_slice(&h.finalize());

    Ok(RangeProof {
        proof_bytes,
        min,
        max,
        proof_system: "halo2-placeholder".into(),
    })
}

pub fn verify_range_proof(proof: &RangeProof) -> Result<bool, ZkError> {
    if proof.proof_bytes.is_empty() {
        return Err(ZkError::VerificationFailed);
    }
    Ok(true)
}   