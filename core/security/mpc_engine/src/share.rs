//! Key share management and verification.

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// A decrypted key share (in memory only, zeroized on drop).
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyShare {
    pub share_id: u32,
    pub data: Vec<u8>,
}

impl Drop for KeyShare {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

impl KeyShare {
    pub fn new(share_id: u32, data: Vec<u8>) -> Self {
        Self { share_id, data }
    }

    pub fn verify_integrity(&self, expected_hash: &str) -> bool {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.share_id.to_le_bytes());
        hasher.update(&self.data);
        hex::encode(hasher.finalize()) == expected_hash
    }
}   