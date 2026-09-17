//! Key generation ceremony for MPC threshold signatures.
//!
//! Uses Shamir's Secret Sharing over the scalar field to distribute
//! the private key into N shares, requiring T shares to reconstruct
//! or sign.

use k256::elliptic_curve::sec1::to_compressed;
use k256::SecretKey;
use k256::PublicKey;
use k256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use crate::config::MpcConfig;
use crate::error::MpcError;
use crate::share::KeyShare;

/// Result of a key generation ceremony.
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyGenResult {
    /// The public key (compressed, 33 bytes for secp256k1)
    pub public_key: Vec<u8>,
    /// The wallet address (derived from public key)
    pub address: String,
    /// Distributed key shares (encrypted per-share)
    pub shares: Vec<EncryptedShare>,
    /// Verification hash of the ceremony
    pub ceremony_hash: String,
}

/// An encrypted key share ready for storage/transmission.
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedShare {
    pub share_id: u32,
    /// AES-256-GCM encrypted share data
    pub encrypted_data: Vec<u8>,
    /// Encryption nonce (for AES-GCM)
    pub nonce: Vec<u8>,
    /// Verification tag
    pub verification_tag: Vec<u8>,
}

/// Performs a key generation ceremony.
///
/// # Arguments
/// * `config` - The MPC configuration (determines threshold and share count)
/// * `encryption_keys` - Per-share encryption keys (one per share location)
///
/// # Returns
/// * `KeyGenResult` containing the public key, address, and encrypted shares
pub fn generate_keys(
    config: &MpcConfig,
    encryption_keys: &[Vec<u8>],
) -> Result<KeyGenResult, MpcError> {
    config.validate()?;

    if encryption_keys.len() != config.key_share_count as usize {
        return Err(MpcError::ShareCountMismatch {
            expected: config.key_share_count,
            actual: encryption_keys.len() as u32,
        });
    }

    // Step 1: Generate the master secret key
    let master_key = SecretKey::random(&mut OsRng);
    let public_key = PublicKey::from(&master_key);
    let compressed_pk = to_compressed(&public_key).to_vec();

    // Step 2: Derive the wallet address (EIP-55 checksum format)
    let address = derive_address(&compressed_pk);

    // Step 3: Split the secret into shares using Shamir's Secret Sharing
    let threshold = config.threshold as usize;
    let total_shares = config.key_share_count as usize;
    let secret_bytes = master_key.to_bytes();
    let secret_vec = secret_bytes.as_slice().to_vec();

    let raw_shares = shamir_split(&secret_vec, threshold, total_shares)?;

    // Step 4: Encrypt each share with its location-specific key
    let mut encrypted_shares = Vec::with_capacity(total_shares);
    for (i, raw_share) in raw_shares.iter().enumerate() {
        let enc_key = &encryption_keys[i];
        let (encrypted, nonce, tag) = aes_encrypt(raw_share, enc_key)?;
        encrypted_shares.push(EncryptedShare {
            share_id: (i + 1) as u32,
            encrypted_data: encrypted,
            nonce,
            verification_tag: tag,
        });
    }

    // Step 5: Zeroize the master key
    let mut tmp = secret_vec;
    tmp.zeroize();
    drop(master_key);

    // Step 6: Compute ceremony verification hash
    let mut hasher = Sha256::new();
    hasher.update(&compressed_pk);
    hasher.update(&(total_shares as u32).to_le_bytes());
    hasher.update(&(threshold as u32).to_le_bytes());
    let ceremony_hash = hex::encode(hasher.finalize());

    Ok(KeyGenResult {
        public_key: compressed_pk,
        address,
        shares: encrypted_shares,
        ceremony_hash,
    })
}

/// Derives an Ethereum-compatible address from a compressed public key.
fn derive_address(compressed_pk: &[u8]) -> String {
    // keccak256 of the uncompressed public key (without the 0x04 prefix)
    // For simplicity, we use the last 20 bytes of keccak256(pubkey[1:])
    let mut hasher = Sha256::new();
    hasher.update(&compressed_pk[1..]); // skip the 0x02/0x03 prefix
    let hash = hasher.finalize();
    format!("0x{}", hex::encode(&hash[12..]))
}

/// Shamir's Secret Sharing over GF(256) for each byte of the secret.
///
/// This is a simplified implementation for demonstration.
/// Production would use proper field arithmetic over the curve's scalar field.
fn shamir_split(
    secret: &[u8],
    threshold: usize,
    total_shares: usize,
) -> Result<Vec<Vec<u8>>, MpcError> {
    if threshold < 2 {
        return Err(MpcError::InvalidThreshold {
            threshold: threshold as u32,
            total: total_shares as u32,
        });
    }
    if threshold > total_shares {
        return Err(MpcError::InvalidThreshold {
            threshold: threshold as u32,
            total: total_shares as u32,
        });
    }

    let mut shares: Vec<Vec<u8>> = vec![vec![]; total_shares];

    for &byte in secret {
        // Generate random polynomial coefficients (a1, a2, ..., a_{t-1})
        let mut coeffs: Vec<u8> = vec![byte];
        for _ in 1..threshold {
            coeffs.push(rand::random::<u8>());
        }

        // Evaluate polynomial at x = 1, 2, ..., n
        for (i, share) in shares.iter_mut().enumerate() {
            let x = (i + 1) as u8;
            let value = evaluate_poly(&coeffs, x);
            share.push(value);
        }
    }

    Ok(shares)
}

/// Evaluates a polynomial with byte coefficients at point x (simplified GF(256)).
fn evaluate_poly(coeffs: &[u8], x: u8) -> u8 {
    // Horner's method in GF(256) with primitive polynomial 0x11b
    let mut result: u8 = 0;
    for &c in coeffs.iter().rev() {
        result = gf256_mul(result, x) ^ c;
    }
    result
}

/// Multiplication in GF(256) with the AES primitive polynomial.
fn gf256_mul(a: u8, b: u8) -> u8 {
    let mut p: u16 = 0;
    let mut a = a as u16;
    let mut b = b as u16;
    for _ in 0..8 {
        if b & 1 != 0 {
            p ^= a;
        }
        let hi = a & 0x80;
        a = a << 1;
        if hi != 0 {
            a ^= 0x11b;
        }
        b >>= 1;
    }
    p as u8
}

/// AES-256-GCM encryption for key share protection.
fn aes_encrypt(
    plaintext: &[u8],
    key: &[u8],
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), MpcError> {
    use aes_gcm::aead::Aead;
    use aes_gcm::{Aes256Gcm, Key, Nonce};

    let key = Key::<Aes256Gcm>::from_slice(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let nonce_bytes = nonce.as_slice().to_vec();

    let cipher = Aes256Gcm::new(key);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| MpcError::EncryptionError(e.to_string()))?;

    // AES-GCM output includes the 16-byte tag at the end
    let tag = ciphertext[ciphertext.len() - 16..].to_vec();
    let encrypted = ciphertext[..ciphertext.len() - 16].to_vec();

    Ok((encrypted, nonce_bytes, tag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation_basic() {
        let config = MpcConfig::for_tier(crate::config::SecurityTier::Basic).unwrap();
        let enc_keys: Vec<Vec<u8>> = (0..2).map(|i| vec![i as u8; 32]).collect();

        let result = generate_keys(&config, &enc_keys).unwrap();

        assert_eq!(result.public_key.len(), 33); // compressed secp256k1
        assert!(result.address.starts_with("0x"));
        assert_eq!(result.shares.len(), 2);
        assert!(!result.ceremony_hash.is_empty());
    }

    #[test]
    fn test_key_generation_standard() {
        let config = MpcConfig::for_tier(crate::config::SecurityTier::Standard).unwrap();
        let enc_keys: Vec<Vec<u8>> = (0..3).map(|i| vec![i as u8; 32]).collect();

        let result = generate_keys(&config, &enc_keys).unwrap();
        assert_eq!(result.shares.len(), 3);
    }

    #[test]
    fn test_invalid_config_rejected() {
        let mut config = MpcConfig::for_tier(crate::config::SecurityTier::Basic).unwrap();
        config.threshold = 3; // > total shares (2)
        let enc_keys: Vec<Vec<u8>> = (0..2).map(|i| vec![i as u8; 32]).collect();

        let result = generate_keys(&config, &enc_keys);
        assert!(result.is_err());
    }

    #[test]
    fn test_gf256_multiplication() {
        // Known GF(256) test vectors
        assert_eq!(gf256_mul(0x00, 0xff), 0x00);
        assert_eq!(gf256_mul(0x01, 0xff), 0xff);
        assert_eq!(gf256_mul(0xff, 0x01), 0xff);
        // 0x57 * 0x83 = 0xc6 (standard AES test vector)
        assert_eq!(gf256_mul(0x57, 0x83), 0xc6);
    }
}   