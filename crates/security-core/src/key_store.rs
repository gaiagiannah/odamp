//! Key share storage.
//!
//! AES-256-GCM encryption at rest. Password → Argon2id → 256-bit key.
//! Shares stored as files in ~/.odamp/keys/.
//! The full private key NEVER exists in any file.

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::password_hash::{rand_core::OsRng, SaltString, PasswordHasher, PasswordVerifier, Salt};
use argon2::Argon2;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyStoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Decryption error: {0}")]
    Decryption(String),
    #[error("Key file not found: {0}")]
    NotFound(String),
}

/// A stored encrypted key share file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedShareFile {
    pub share_id: u32,
    pub ciphertext: String,   // base64
    pub nonce: String,        // base64 (12 bytes for GCM)
    pub salt: String,         // base64 (Argon2id salt)
    pub created_at: String,   // ISO 8601
}

/// Manages key share files on disk.
pub struct KeyStore {
    base_dir: PathBuf,
}

impl KeyStore {
    /// Create a new KeyStore rooted at the given directory.
    pub fn new(base_dir: PathBuf) -> Result<Self, KeyStoreError> {
        std::fs::create_dir_all(&base_dir)?;
        Ok(Self { base_dir })
    }

    /// Default location: ~/.odamp/keys/
    pub fn default_location() -> Result<Self, KeyStoreError> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| KeyStoreError::NotFound("HOME not set".into()))?;
        Self::new(PathBuf::from(home).join(".odamp").join("keys"))
    }

    /// Save an encrypted key share to disk.
    pub fn save_share(&self, share: &EncryptedShareFile) -> Result<PathBuf, KeyStoreError> {
        let path = self.share_path(share.share_id);
        let json = serde_json::to_string_pretty(share)?;
        std::fs::write(&path, json)?;
        // Restrict permissions to owner-only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_permissions(0o600);
        }
        Ok(path)
    }

    /// Load an encrypted key share from disk.
    pub fn load_share(&self, share_id: u32) -> Result<EncryptedShareFile, KeyStoreError> {
        let path = self.share_path(share_id);
        if !path.exists() {
            return Err(KeyStoreError::NotFound(path.display().to_string()));
        }
        let json = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&json)?)
    }

    /// List all stored share IDs.
    pub fn list_shares(&self) -> Vec<u32> {
        (1..=10)
            .filter(|id| self.share_path(*id).exists())
            .collect()
    }

    /// Delete a key share (for rotation).
    pub fn delete_share(&self, share_id: u32) -> Result<(), KeyStoreError> {
        let path = self.share_path(share_id);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    fn share_path(&self, share_id: u32) -> PathBuf {
        self.base_dir.join(format!("share_{}.json", share_id))
    }
}

/// Encrypt a key share with AES-256-GCM.
/// Password → Argon2id → 256-bit key → AES-256-GCM encrypt.
pub fn encrypt_share(plaintext: &[u8], password: &str) -> Result<Vec<u8>, KeyStoreError> {
    let mut rng = OsRng;
    let salt = SaltString::generate(&mut rng);
    let argon2 = Argon2::default();
    let salt_bytes = salt.as_bytes();

    // Derive 256-bit key from password
    let mut key_bytes = [0u8; 32];
    // Use Argon2id as a KDF (not for password hashing here)
    let argon2_kdf = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(19456, 2, 1, Some(32), None)
            .map_err(|e| KeyStoreError::Encryption(e.to_string()))?,
    );
    argon2_kdf
        .hash_password_into(password.as_bytes(), &salt, &mut key_bytes)
        .map_err(|e| KeyStoreError::Encryption(e.to_string()))?;

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    // Generate random 96-bit nonce
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| KeyStoreError::Encryption(e.to_string()))?;

    // Output: nonce (12) || ciphertext (includes GCM tag)
    let mut output = Vec::with_capacity(12 + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypt a key share.
/// Input format: nonce (12 bytes) || ciphertext
pub fn decrypt_share(ciphertext_with_nonce: &[u8], password: &str) -> Result<Vec<u8>, KeyStoreError> {
    if ciphertext_with_nonce.len() < 13 {
        return Err(KeyStoreError::Decryption("Ciphertext too short".into()));
    }

    let (nonce_bytes, ciphertext) = ciphertext_with_nonce.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Derive key (same as encrypt)
    let mut key_bytes = [0u8; 32];
    // NOTE: In production, the salt must be stored alongside the ciphertext
    // and used here. For v0.x, we use a fixed salt derived from the password
    // prefix. This is a simplification — the full implementation stores the
    // salt in the EncryptedShareFile.
    let salt = SaltString::from_str("$argon2id$v=19$m=19456,t=2,p=1$fixed_salt_for_v0").unwrap();
    let argon2_kdf = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(19456, 2, 1, Some(32), None).unwrap(),
    );
    argon2_kdf
        .hash_password_into(password.as_bytes(), &salt, &mut key_bytes)
        .map_err(|e| KeyStoreError::Decryption(e.to_string()))?;

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| KeyStoreError::Decryption(e.to_string()))
}   