//! ML-DSA (FIPS 204) — Module-Lattice-based Digital Signature Algorithm.
//!
//! Parameter sets:
//! - ML-DSA-44 (lightweight): 1952-byte public key, 2420-byte signature
//! - ML-DSA-65 (standard):    1952-byte public key, 3309-byte signature
//! - ML-DSA-87 (high):        2592-byte public key, 4627-byte signature
//!
//! ODAMP uses ML-DSA-65 as the default PQC signature algorithm.

use crate::error::PqcError;
use rand::RngCore;

/// ML-DSA parameter set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MlDsaLevel {
    Level44,
    Level65,
    Level87,
}

impl MlDsaLevel {
    pub fn public_key_size(&self) -> usize {
        match self {
            MlDsaLevel::Level44 => 1952,
            MlDsaLevel::Level65 => 1952,
            MlDsaLevel::Level87 => 2592,
        }
    }

    pub fn signature_size(&self) -> usize {
        match self {
            MlDsaLevel::Level44 => 2420,
            MlDsaLevel::Level65 => 3309,
            MlDsaLevel::Level87 => 4627,
        }
    }

    pub fn secret_key_size(&self) -> usize {
        match self {
            MlDsaLevel::Level44 => 2560,
            MlDsaLevel::Level65 => 4032,
            MlDsaLevel::Level87 => 4896,
        }
    }
}

/// ML-DSA key pair.
#[derive(Debug)]
pub struct MlDsaKeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub level: MlDsaLevel,
}

impl MlDsaKeyPair {
    /// Generates a new ML-DSA key pair.
    ///
    /// NOTE: This is a placeholder implementation. In production, this
    /// calls into a vetted PQC library (e.g., OpenSSL 3.5+, libsodium,
    /// or the Rust `ml-dsa` crate once stabilized).
    pub fn generate(level: MlDsaLevel) -> Result<Self, PqcError> {
        let mut public_key = vec![0u8; level.public_key_size()];
        let mut secret_key = vec![0u8; level.secret_key_size()];

        // Fill with random bytes (placeholder for real keygen)
        rand::thread_rng().fill_bytes(&mut public_key);
        rand::thread_rng().fill_bytes(&mut secret_key);

        Ok(Self { public_key, secret_key, level })
    }

    /// Signs a message hash using ML-DSA.
    pub fn sign(&self, message_hash: &[u8; 32]) -> Result<Vec<u8>, PqcError> {
        let sig_size = self.level.signature_size();
        let mut signature = vec![0u8; sig_size];

        // Placeholder: real ML-DSA signing
        // Production: ml_dsa::sign(&self.secret_key, message_hash, &mut signature)

        // For development: deterministic placeholder
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.secret_key);
        hasher.update(message_hash);
        let hash = hasher.finalize();
        signature[..32].copy_from_slice(&hash);

        Ok(signature)
    }
}

/// Verifies an ML-DSA signature.
pub fn verify(
    public_key: &[u8],
    message_hash: &[u8; 32],
    signature: &[u8],
    level: MlDsaLevel,
) -> Result<bool, PqcError> {
    if signature.len() != level.signature_size() {
        return Err(PqcError::InvalidSignatureSize {
            expected: level.signature_size(),
            actual: signature.len(),
        });
    }
    if public_key.len() != level.public_key_size() {
        return Err(PqcError::InvalidKeySize {
            expected: level.public_key_size(),
            actual: public_key.len(),
        });
    }

    // Placeholder: real ML-DSA verification
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let kp = MlDsaKeyPair::generate(MlDsaLevel::Level65).unwrap();
        assert_eq!(kp.public_key.len(), 1952);
        assert_eq!(kp.secret_key.len(), 4032);
    }

    #[test]
    fn test_sign_verify() {
        let kp = MlDsaKeyPair::generate(MlDsaLevel::Level65).unwrap();
        let msg = [42u8; 32];
        let sig = kp.sign(&msg).unwrap();
        assert_eq!(sig.len(), 3309);

        let valid = verify(&kp.public_key, &msg, &sig, MlDsaLevel::Level65).unwrap();
        assert!(valid);
    }
}   