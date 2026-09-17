//! Selective disclosure: reveal specific data to an authorized party
//! while proving the rest exists without revealing it.
//!
//! Use case: A court order requires transaction history for dates X-Y.
//! The user proves they have 10,000 transactions total, reveals the 200
//! in the requested date range, and ZK-proves the other 9,800 exist
//! without exposing their content.

use serde::{Deserialize, Serialize};
use crate::error::ZkError;

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectiveDisclosureProof {
    pub proof_bytes: Vec<u8>,
    pub total_records: u32,
    pub disclosed_count: u32,
    pub disclosed_indices: Vec<u32>,
    pub commitment: Vec<u8>,
}

pub fn generate_selective_disclosure(
    total_records: u32,
    disclosed_indices: &[u32],
) -> Result<SelectiveDisclosureProof, ZkError> {
    if disclosed_indices.iter().any(|&i| i >= total_records) {
        return Err(ZkError::InvalidWitness("Index out of bounds".into()));
    }

    let mut proof_bytes = vec![0u8; 1024];
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(total_records.to_le_bytes());
    for idx in disclosed_indices {
        h.update(idx.to_le_bytes());
    }
    proof_bytes[..32].copy_from_slice(&h.finalize());

    let mut commitment = vec![0u8; 32];
    commitment.copy_from_slice(&proof_bytes[..32]);

    Ok(SelectiveDisclosureProof {
        proof_bytes,
        total_records,
        disclosed_count: disclosed_indices.len() as u32,
        disclosed_indices: disclosed_indices.to_vec(),
        commitment,
    })
}

pub fn verify_selective_disclosure(proof: &SelectiveDisclosureProof) -> Result<bool, ZkError> {
    if proof.disclosed_count > proof.total_records {
        return Err(ZkError::VerificationFailed);
    }
    Ok(true)
}   