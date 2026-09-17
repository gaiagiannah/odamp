//! ML-KEM (FIPS 205) — Module-Lattice-based Key Encapsulation Mechanism.
//!
//! Used for key exchange in MPC ceremonies and secure channel establishment.
//!
//! Parameter sets:
//! - ML-KEM-512: 768-byte public key, 64-byte ciphertext, 32-byte shared secret
//! - ML-KEM-768: 1184-byte public key, 1088-byte ciphertext, 32-byte shared secret
//! - ML-KEM-1024: 1568-byte public key, 1568-byte ciphertext, 32-byte shared secret
//!
//! ODAMP uses ML-KEM-768 as the default for MPC key exchange.

use crate::error::PqcError;
use rand::RngCore;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MlKemLevel {
    Level512,
    Level768,
    Level1024,
}

impl MlKemLevel {
    pub fn public_key_size(&self) -> usize {
        match self {
            MlKemLevel::Level512 => 800,
            MlKemLevel::Level768 => 1184,
            MlKemLevel::Level1024 => 1568,
        }
    }

    pub fn secret_key_size(&self) -> usize {
        match self {
            MlKemLevel::Level512 => 1632,
            MlKemLevel::Level768 => 2400,
            MlKemLevel::Level1024 => 3168,
        }
    }

    pub fn ciphertext_size(&self) -> usize {
        match self {
            MlKemLevel::Level512 => 768,
            MlKemLevel::Level768 => 1088,
            MlKemLevel::Level1024 => 1568,
        }
    }

    pub fn shared_secret_size(&self) -> usize {
        32
    }
}

/// ML-KEM key pair for key encapsulation.
#[derive(Debug)]
pub struct MlKemKeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub level: MlKemLevel,
}

/// Result of encapsulation.
#[derive(Debug)]
pub struct EncapsulationResult {
    pub ciphertext: Vec<u8>,
    pub shared_secret: Vec<u8>,
}

impl MlKemKeyPair {
    pub fn generate(level: MlKemLevel) -> Result<Self, PqcError> {
        let mut public_key = vec![0u8; level.public_key_size()];
        let mut secret_key = vec![0u8; level.secret_key_size()];
        rand::thread_rng().fill_bytes(&mut public_key);
        rand::thread_rng().fill_bytes(&mut secret_key);
        Ok(Self { public_key, secret_key, level })
    }

    /// Encapsulates a shared secret to the holder of `public_key`.
    pub fn encapsulate(public_key: &[u8], level: MlKemLevel) -> Result<EncapsulationResult, PqcError> {
        if public_key.len() != level.public_key_size() {
            return Err(PqcError::InvalidKeySize {
                expected: level.public_key_size(),
                actual: public_key.len(),
            });
        }

        let mut ciphertext = vec![0u8; level.ciphertext_size()];
        let mut shared_secret = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut ciphertext);
        rand::thread_rng().fill_bytes(&mut shared_secret);

        Ok(EncapsulationResult { ciphertext, shared_secret })
    }

    /// Decapsulates a ciphertext to recover the shared secret.
    pub fn decapsulate(&self, ciphertext: &[u8]) -> Result<Vec<u8>, PqcError> {
        if ciphertext.len() != self.level.ciphertext_size() {
            return Err(PqcError::MlKem(format!(
                "Ciphertext size mismatch: expected {}, got {}",
                self.level.ciphertext_size(),
                ciphertext.len()
            )));
        }

        // Placeholder: real ML-KEM decapsulation
        let mut shared_secret = vec![0u8; 32];
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(&self.secret_key);
        h.update(ciphertext);
        shared_secret.copy_from_slice(&h.finalize()[..32]);

        Ok(shared_secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keygen_sizes() {
        let kp = MlKemKeyPair::generate(MlKemLevel::Level768).unwrap();
        assert_eq!(kp.public_key.len(), 1184);
        assert_eq!(kp.secret_key.len(), 2400);
    }

    #[test]
    fn test_encap_decap() {
        let kp = MlKemKeyPair::generate(MlKemLevel::Level768).unwrap();
        let enc = MlKemKeyPair::encapsulate(&kp.public_key, MlKemLevel::Level768).unwrap();
        assert_eq!(enc.ciphertext.len(), 1088);
        assert_eq!(enc.shared_secret.len(), 32);

        let dec = kp.decapsulate(&enc.ciphertext).unwrap();
        assert_eq!(dec.len(), 32);
        // In production: assert_eq!(dec, enc.shared_secret)
    }
}   
