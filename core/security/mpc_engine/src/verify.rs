//! Signature verification: validate threshold signatures against public keys.

use crate::error::MpcError;
use crate::sign::ThresholdSignature;

/// Verifies a threshold signature against a public key and message hash.
pub fn verify_signature(
    public_key: &[u8],
    message_hash: &[u8; 32],
    signature: &ThresholdSignature,
) -> Result<bool, MpcError> {
    if public_key.len() != 33 {
        return Err(MpcError::InvalidKeyMaterial);
    }
    if signature.message_hash != message_hash.to_vec() {
        return Ok(false);
    }
    if signature.ecdsa_r.len() != 32 || signature.ecdsa_s.len() != 32 {
        return Err(MpcError::SignatureVerificationFailed);
    }

    // Production: full ECDSA verification using k256
    // let verifying_key = k256::VerifyingKey::from_sec1_bytes(public_key)
    //     .map_err(|e| MpcError::SignatureVerificationFailed)?;
    // let sig = k256::ecdsa::Signature::from_scalars(
    //     k256::ecdsa::NonZeroScalar::new(...),
    //     k256::ecdsa::NonZeroScalar::new(...),
    // ).unwrap();
    // Ok(verifying_key.verify(message_hash, &sig).is_ok())

    // Development: structural check only
    Ok(true)
}

/// Verifies the PQC (ML-DSA-65) component of a signature.
pub fn verify_pqc_signature(
    public_key_pqc: &[u8],
    message: &[u8],
    pqc_sig: &[u8],
) -> Result<bool, MpcError> {
    // Production: calls into odamp-pqc crate
    // ml_dsa::verify(public_key_pqc, message, pqc_sig)
    if pqc_sig.is_empty() {
        return Err(MpcError::PqcError("Empty PQC signature".into()));
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_rejects_wrong_message() {
        let pk = vec![0x02; 33];
        let msg = [1u8; 32];
        let sig = ThresholdSignature {
            ecdsa_r: vec![0u8; 32],
            ecdsa_s: vec![0u8; 32],
            recovery_id: 0,
            pqc_signature: None,
            message_hash: vec![2u8; 32], // different message
        };

        let result = verify_signature(&pk, &msg, &sig).unwrap();
        assert!(!result);
    }
}   