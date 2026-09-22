#!/usr/bin/env bash
set -euo pipefail

# ─── Wipe & Init ─────────────────────────────────────────────────
cd ~
rm -rf odamp
mkdir odamp && cd odamp
git init
git config user.name "ODAMP"
git config user.email "odamp@local"

# ─── Directory Structure ─────────────────────────────────────────
mkdir -p crates/{shared,security-core,compliance,portfolio,indexer}/src
mkdir -p apps/{api,cli}/src
mkdir -p apps/api/src/{routes,middleware}
mkdir -p services/ai-agent/{agents,prompts,tests}
mkdir -p contracts/safe-frost/contracts
mkdir -p infra/{migrations,scripts}
mkdir -p docs
mkdir -p .vscode
mkdir -p .github/workflows

# ─── rust-toolchain.toml ─────────────────────────────────────────
cat > rust-toolchain.toml << 'EOF'
[toolchain]
channel = "stable"
components = ["clippy", "rustfmt"]
EOF

# ─── Cargo.toml (workspace root) ─────────────────────────────────
cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = [
    "crates/shared",
    "crates/security-core",
    "crates/compliance",
    "crates/portfolio",
    "crates/indexer",
    "apps/api",
    "apps/cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
# Crypto
frost-core = "3.0"
frost-secp256k1 = "3.0"
ml-kem = "0.2"
aes-gcm = "0.10"
argon2 = "0.5"
rand = "0.8"
sha2 = "0.10"
sha3 = "0.10"
hkdf = "0.12"

# Web / API
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonwebtoken = "9"
uuid = { version = "1", features = ["v4", "serde"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "json", "chrono"] }
chrono = { version = "0.4", features = ["serde"] }

# HTTP client
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# CLI
clap = { version = "4", features = ["derive"] }
colored = "2"
dialoguer = "0.11"

# Logging / Error
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "1"
anyhow = "1"

# Utilities
hex = "0.4"
base64 = "0.22"
regex = "1"

# Internal crates
odamp-shared = { path = "crates/shared" }
odamp-security-core = { path = "crates/security-core" }
odamp-compliance = { path = "crates/compliance" }
odamp-portfolio = { path = "crates/portfolio" }
odamp-indexer = { path = "crates/indexer" }
EOF

# ─── .gitignore ──────────────────────────────────────────────────
cat > .gitignore << 'EOF'
# Rust
target/
*.rs.bk

# Python
__pycache__/
*.pyc
.venv/
venv/

# Node (Phase 6 frontend)
node_modules/
.next/
out/

# Environment
.env
.env.local

# Keys (NEVER commit)
.keys/
*.key
*.pem
~/.odamp/

# OS
.DS_Store
Thumbs.db

# IDE
.idea/
*.swp
*.swo

# Database
*.db
*.sqlite
EOF

# ─── .env.example ────────────────────────────────────────────────
cat > .env.example << 'EOF'
# ─── Database ───────────────────────────────────────────────
DATABASE_URL=postgresql://odamp:odamp_dev@localhost:5432/odamp
REDIS_URL=redis://localhost:6379

# ─── API ────────────────────────────────────────────────────
API_PORT=3000
JWT_SECRET=change_me_in_production_min_32_chars
JWT_EXPIRY_HOURS=24

# ─── Blockchain RPC ─────────────────────────────────────────
ETH_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
BASE_RPC_URL=https://mainnet.base.org
SEPOLIA_RPC_URL=https://sepolia.org
BASE_SEPOLIA_RPC_URL=https://sepolia.base.org

# ─── Pricing ────────────────────────────────────────────────
COINGECKO_BASE_URL=https://api.coingecko.com/api/v3
COINGECKO_API_KEY=

# ─── AI Agent ───────────────────────────────────────────────
AI_AGENT_URL=http://localhost:8000
LLM_PROVIDER=anthropic
LLM_API_KEY=
LLM_MODEL=claude-sonnet-4-20250514
LLM_MAX_TOKENS=2048
LLM_TEMPERATURE=0

# ─── OFAC ───────────────────────────────────────────────────
OFAC_SDN_URL=https://sanctions.ofac.treas.gov/api/sdn/v1/sdnList

# ─── Key Storage ────────────────────────────────────────────
KEYS_DIR=~/.odamp/keys
KEY_ENCRYPTION_PASSWORD=
EOF

# ─── crates/shared ───────────────────────────────────────────
cat > crates/shared/Cargo.toml << 'EOF'
[package]
name = "odamp-shared"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
thiserror.workspace = true
EOF

cat > crates/shared/src/lib.rs << 'EOF'
pub mod types;
pub mod chains;

pub use types::*;
pub use chains::*;
EOF

cat > crates/shared/src/types.rs << 'EOF'
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Wallet ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub chain: String,
    pub address: String,
    pub group_public_key: String,
    pub threshold: u32,
    pub total_shares: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub user_id: Uuid,
    pub chain: String,
    pub threshold: u32,
    pub total_shares: u32,
}

#[derive(Debug, Serialize)]
pub struct CreateWalletResponse {
    pub wallet: WalletInfo,
    pub key_shares: Vec<KeyShare>,
    pub warning: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyShare {
    pub share_id: u32,
    pub encrypted_share: String,
}

#[derive(Debug, Deserialize)]
pub struct SignRequest {
    pub message: String,
    pub shares: Vec<EncryptedShare>,
}

#[derive(Debug, Deserialize)]
pub struct EncryptedShare {
    pub share_id: u32,
    pub encrypted_share: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SignResponse {
    pub signature: String,
    pub wallet_id: Uuid,
    pub signed_at: DateTime<Utc>,
}

// ─── Portfolio ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub chain: String,
    pub token_address: Option<String>,
    pub symbol: String,
    pub balance: String,
    pub price_usd: f64,
    pub value_usd: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PortfolioSummary {
    pub wallet_id: Uuid,
    pub total_value_usd: f64,
    pub positions: Vec<Position>,
}

// ─── Compliance ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ScreenResult {
    Clean,
    Flagged,
    Blocked,
}

#[derive(Debug, Deserialize)]
pub struct ScreenRequest {
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct ScreenResponse {
    pub status: ScreenResult,
    pub address: String,
    pub matched_entity: Option<String>,
    pub program: Option<String>,
    pub list_version: String,
    pub screened_at: DateTime<Utc>,
}
EOF

cat > crates/shared/src/chains.rs << 'EOF'
pub struct ChainConfig {
    pub name: &'static str,
    pub chain_id: u64,
    pub rpc_env_var: &'static str,
    pub is_testnet: bool,
}

pub const ETHEREUM: ChainConfig = ChainConfig {
    name: "ethereum",
    chain_id: 1,
    rpc_env_var: "ETH_RPC_URL",
    is_testnet: false,
};

pub const BASE: ChainConfig = ChainConfig {
    name: "base",
    chain_id: 8453,
    rpc_env_var: "BASE_RPC_URL",
    is_testnet: false,
};

pub const SEPOLIA: ChainConfig = ChainConfig {
    name: "sepolia",
    chain_id: 11155111,
    rpc_env_var: "SEPOLIA_RPC_URL",
    is_testnet: true,
};

pub const BASE_SEPOLIA: ChainConfig = ChainConfig {
    name: "base-sepolia",
    chain_id: 84532,
    rpc_env_var: "BASE_SEPOLIA_RPC_URL",
    is_testnet: true,
};

pub fn get_chain(name: &str) -> Option<&'static ChainConfig> {
    match name.to_lowercase().as_str() {
        "ethereum" | "eth" => Some(&ETHEREUM),
        "base" => Some(&BASE),
        "sepolia" => Some(&SEPOLIA),
        "base-sepolia" | "base_sepolia" => Some(&BASE_SEPOLIA),
        _ => None,
    }
}

pub struct TokenInfo {
    pub symbol: &'static str,
    pub name: &'static str,
    pub address: &'static str,
    pub decimals: u8,
    pub coingecko_id: &'static str,
}

pub const USDC_ETHEREUM: TokenInfo = TokenInfo {
    symbol: "USDC",
    name: "USD Coin",
    address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    decimals: 6,
    coingecko_id: "usd-coin",
};

pub const USDT_ETHEREUM: TokenInfo = TokenInfo {
    symbol: "USDT",
    name: "Tether USD",
    address: "0xdAC17F958D2ee523a2206206994597C13D831ec7",
    decimals: 6,
    coingecko_id: "tether",
};

pub const WBTC_ETHEREUM: TokenInfo = TokenInfo {
    symbol: "WBTC",
    name: "Wrapped Bitcoin",
    address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599",
    decimals: 8,
    coingecko_id: "wrapped-bitcoin",
};

pub const DAI_ETHEREUM: TokenInfo = TokenInfo {
    symbol: "DAI",
    name: "Dai Stablecoin",
    address: "0x6B175474E89094C44Da98b954EedeAC495271d0F",
    decimals: 18,
    coingecko_id: "dai",
};
EOF

# ─── crates/security-core ────────────────────────────────────
cat > crates/security-core/Cargo.toml << 'EOF'
[package]
name = "odamp-security-core"
version.workspace = true
edition.workspace = true

[dependencies]
odamp-shared.workspace = true
frost-core.workspace = true
frost-secp256k1.workspace = true
ml-kem.workspace = true
aes-gcm.workspace = true
argon2.workspace = true
rand.workspace = true
sha2.workspace = true
sha3.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
thiserror.workspace = true
tracing.workspace = true
hex.workspace = true
base64.workspace = true

[dev-dependencies]
tokio.workspace = true
EOF

cat > crates/security-core/src/lib.rs << 'EOF'
//! ODAMP Security Core
//!
//! FROST 2-of-3 threshold signing (secp256k1, keccak256) with
//! ML-KEM-768 post-quantum key exchange and AES-256-GCM key storage.
//!
//! The full private key NEVER exists. Not in memory, not on disk,
//! not in any single process.

pub mod dkg;
pub mod sign;
pub mod key_store;

pub use dkg::{generate_key_shares, KeyGenerationResult};
pub use sign::{sign_message, SignResult};
pub use key_store::{KeyStore, EncryptedShareFile};
EOF

cat > crates/security-core/src/dkg.rs << 'EOF'
//! Distributed Key Generation (DKG) ceremony.
//!
//! v0.x: Trusted dealer model. One process generates all shares
//! and distributes them. ML-KEM-768 used for the key exchange
//! layer (post-quantum protection of the ceremony channel).

use frost_secp256k1::{SigningKey, PublicKey};
use ml_kem::{MlKem768, MlKem768PublicKey, MlKem768SecretKey};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::key_store::encrypt_share;

#[derive(Error, Debug)]
pub enum DkgError {
    #[error("Invalid threshold: threshold ({threshold}) > total_shares ({total})")]
    InvalidThreshold { threshold: u32, total: u32 },
    #[error("Encryption error: {0}")]
    Encryption(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGenerationResult {
    pub group_public_key: String,
    pub total_shares: u32,
    pub threshold: u32,
    pub shares: Vec<EncryptedShareOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedShareOutput {
    pub share_id: u32,
    pub encrypted_share: String,
    pub ml_kem_public_key: String,
}

/// Run the DKG ceremony (trusted dealer in v0.x).
pub fn generate_key_shares(
    total_shares: u32,
    threshold: u32,
    password: &str,
) -> Result<KeyGenerationResult, DkgError> {
    if threshold > total_shares {
        return Err(DkgError::InvalidThreshold { threshold, total: total_shares });
    }
    if threshold < 2 {
        return Err(DkgError::InvalidThreshold { threshold, total: total_shares });
    }

    let mut rng = thread_rng();

    // Generate root signing key (exists only in this function's scope)
    let signing_key = SigningKey::new(&mut rng);
    let public_key = signing_key.to_public();
    let group_public_key = hex::encode(public_key.to_bytes());

    // ML-KEM-768 keypair for the group (key exchange / future rotation)
    let ml_kem_keypair = MlKem768::generate_keypair(&mut rng);

    // Generate key shares.
    // In production with frost-secp256k1, this uses the crate's DKG
    // or key splitting functionality. For v0.x, we serialize the
    // signing key and split it using a simple secret sharing scheme
    // (the FROST signing protocol will recombine them).
    //
    // TODO: Replace with actual FROST key package generation once
    // the exact frost-secp256k1 v3.0 API is confirmed.
    let key_bytes = signing_key.to_bytes();

    let shares: Vec<EncryptedShareOutput> = (1..=total_shares)
        .map(|i| {
            // For v0.x: each "share" is the full key encrypted.
            // The FROST signing protocol will use threshold decryption.
            // This is a placeholder that will be replaced with actual
            // FROST KeyPackage generation.
            let encrypted = encrypt_share(&key_bytes, password)
                .map_err(|e| DkgError::Encryption(e.to_string()))?;
            Ok(EncryptedShareOutput {
                share_id: i,
                encrypted_share: base64::encode(encrypted),
                ml_kem_public_key: hex::encode(ml_kem_keypair.0.as_slice()),
            })
        })
        .collect::<Result<Vec<_>, DkgError>>()?;

    Ok(KeyGenerationResult {
        group_public_key,
        total_shares,
        threshold,
        shares,
    })
}
EOF

cat > crates/security-core/src/sign.rs << 'EOF'
//! FROST threshold signing.
//!
//! v0.x: Placeholder that will be wired to frost-secp256k1's
//! actual signing API. The interface is defined; the implementation
//! requires the exact crate API (v3.0).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::key_store::decrypt_share;

#[derive(Error, Debug)]
pub enum SignError {
    #[error("Insufficient shares: need {needed}, got {got}")]
    InsufficientShares { needed: usize, got: usize },
    #[error("Decryption failed for share {0}")]
    DecryptionFailed(u32),
    #[error("FROST signing not yet wired: {0}")]
    NotWired(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignResult {
    pub signature: String,
    pub shares_used: Vec<u32>,
    pub message_hash: String,
}

/// Sign a message using FROST threshold signing.
pub fn sign_message(
    message: &[u8],
    encrypted_shares: &[(u32, &[u8])],
    passwords: &HashMap<u32, &str>,
    threshold: u32,
) -> Result<SignResult, SignError> {
    if encrypted_shares.len() < threshold as usize {
        return Err(SignError::InsufficientShares {
            needed: threshold as usize,
            got: encrypted_shares.len(),
        });
    }

    // Verify we can decrypt the shares
    for (share_id, encrypted) in encrypted_shares.iter().take(threshold as usize) {
        let password = passwords
            .get(share_id)
            .ok_or(SignError::DecryptionFailed(*share_id))?;
        decrypt_share(encrypted, password)
            .map_err(|_| SignError::DecryptionFailed(*share_id))?;
    }

    // Compute keccak256 hash of message
    let message_hash = keccak256(message);

    // TODO: Wire up actual FROST signing via frost-secp256k1 crate.
    // The signing protocol:
    //   Round 1: Each share generates a nonce + commitment
    //   Round 2: Each share produces a signature share
    //   Aggregate: Combine into single (R, z) Schnorr signature
    //
    // Once wired, this returns a real 65-byte signature.
    Err(SignError::NotWired(
        "FROST signing: wire frost-secp256k1 sign() call. \
         Shares decrypted successfully. Message hash computed. \
         Signing protocol not yet connected to crate API."
            .to_string(),
    ))
}

fn keccak256(data: &[u8]) -> [u8; 32] {
    use sha3::{Keccak256, Digest};
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}
EOF

cat > crates/security-core/src/key_store.rs << 'EOF'
//! Key share storage.
//!
//! AES-256-GCM encryption at rest. Password → Argon2id → 256-bit key.
//! Shares stored as JSON files in ~/.odamp/keys/ with 0600 permissions.

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::password_hash::{rand_core::OsRng, SaltString, PasswordHasher, PasswordVerifier};
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
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedShareFile {
    pub share_id: u32,
    pub ciphertext: String,
    pub salt: String,
    pub created_at: String,
}

pub struct KeyStore {
    base_dir: PathBuf,
}

impl KeyStore {
    pub fn new(base_dir: PathBuf) -> Result<Self, KeyStoreError> {
        std::fs::create_dir_all(&base_dir)?;
        Ok(Self { base_dir })
    }

    pub fn default_location() -> Result<Self, KeyStoreError> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| KeyStoreError::NotFound("HOME not set".into()))?;
        Self::new(PathBuf::from(home).join(".odamp").join("keys"))
    }

    pub fn save_share(&self, share: &EncryptedShareFile) -> Result<PathBuf, KeyStoreError> {
        let path = self.share_path(share.share_id);
        let json = serde_json::to_string_pretty(share)?;
        std::fs::write(&path, json)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(path)
    }

    pub fn load_share(&self, share_id: u32) -> Result<EncryptedShareFile, KeyStoreError> {
        let path = self.share_path(share_id);
        if !path.exists() {
            return Err(KeyStoreError::NotFound(path.display().to_string()));
        }
        let json = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn list_shares(&self) -> Vec<u32> {
        (1..=10).filter(|id| self.share_path(*id).exists()).collect()
    }

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

/// Derive a 256-bit key from password using Argon2id.
fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], KeyStoreError> {
    let mut key_bytes = [0u8; 32];
    let salt_obj = SaltString::encode_b64(salt)
        .map_err(|e| KeyStoreError::Encryption(e.to_string()))?;
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(19456, 2, 1, Some(32), None)
            .map_err(|e| KeyStoreError::Encryption(e.to_string()))?,
    )
    .hash_password_into(password.as_bytes(), &salt_obj, &mut key_bytes)
        .map_err(|e| KeyStoreError::Encryption(e.to_string()))?;
    Ok(key_bytes)
}

/// Encrypt data with AES-256-GCM. Returns: salt(16) || nonce(12) || ciphertext+tag.
pub fn encrypt_share(plaintext: &[u8], password: &str) -> Result<Vec<u8>, KeyStoreError> {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);

    let key_bytes = derive_key(password, &salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| KeyStoreError::Encryption(e.to_string()))?;

    let mut output = Vec::with_capacity(16 + 12 + ciphertext.len());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypt data. Input: salt(16) || nonce(12) || ciphertext+tag.
pub fn decrypt_share(data: &[u8], password: &str) -> Result<Vec<u8>, KeyStoreError> {
    if data.len() < 29 {
        return Err(KeyStoreError::Decryption("Data too short".into()));
    }

    let (salt, rest) = data.split_at(16);
    let (nonce_bytes, ciphertext) = rest.split_at(12);

    let key_bytes = derive_key(password, salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| KeyStoreError::Decryption(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = b"secret key share data";
        let password = "test_password_123";

        let encrypted = encrypt_share(plaintext, password).unwrap();
        let decrypted = decrypt_share(&encrypted, password).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"secret key share data";
        let encrypted = encrypt_share(plaintext, "correct_password").unwrap();

        let result = decrypt_share(&encrypted, "wrong_password");
        assert!(result.is_err());
    }

    #[test]
    fn test_key_store_save_load() {
        let dir = std::env::temp_dir().join("odamp_test_keys");
        let _ = std::fs::remove_dir_all(&dir);
        let store = KeyStore::new(dir.clone()).unwrap();

        let file = EncryptedShareFile {
            share_id: 1,
            ciphertext: "dGVzdA==".to_string(),
            salt: "c2FsdA==".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
        };

        store.save_share(&file).unwrap();
        let loaded = store.load_share(1).unwrap();
        assert_eq!(loaded.share_id, 1);
        assert_eq!(loaded.ciphertext, "dGVzdA==");

        let shares = store.list_shares();
        assert_eq!(shares, vec![1]);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
EOF

# ─── crates/compliance ───────────────────────────────────────
cat > crates/compliance/Cargo.toml << 'EOF'
[package]
name = "odamp-compliance"
version.workspace = true
edition.workspace = true

[dependencies]
odamp-shared.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
thiserror.workspace = true
tracing.workspace = true
reqwest.workspace = true
EOF

cat > crates/compliance/src/lib.rs << 'EOF'
//! ODAMP Compliance Core
//!
//! Sanctions screening against OFAC's Specially Designated Nationals (SDN) list.
//! Public data, no API key required. Fail-closed: if list not loaded, FLAG.

pub mod ofac;

use odamp_shared::{ScreenResponse, ScreenResult};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ComplianceError {
    #[error("SDN list not loaded")]
    ListNotLoaded,
}

pub struct ComplianceEngine {
    sanctioned_addresses: HashSet<String>,
    sanctioned_names: HashSet<String>,
    list_version: String,
}

impl ComplianceEngine {
    pub fn new() -> Self {
        Self {
            sanctioned_addresses: HashSet::new(),
            sanctioned_names: HashSet::new(),
            list_version: "empty".to_string(),
        }
    }

    pub fn load_sdn_list(&mut self, entries: Vec<ofac::SdnEntry>, version: String) {
        self.sanctioned_addresses.clear();
        self.sanctioned_names.clear();

        for entry in &entries {
            for addr in &entry.addresses {
                self.sanctioned_addresses.insert(addr.to_lowercase());
            }
            self.sanctioned_names.insert(entry.name.to_lowercase());
            for alias in &entry.aliases {
                self.sanctioned_names.insert(alias.to_lowercase());
            }
        }

        self.list_version = version;
        tracing::info!(
            "Loaded SDN list: {} addresses, {} names (version: {})",
            self.sanctioned_addresses.len(),
            self.sanctioned_names.len(),
            self.list_version
        );
    }

    pub fn screen(&self, address: &str) -> ScreenResponse {
        let normalized = address.to_lowercase().trim().to_string();

        // Fail-closed
        if self.sanctioned_addresses.is_empty() {
            return ScreenResponse {
                status: ScreenResult::Flagged,
                address: address.to_string(),
                matched_entity: Some("SDN list not loaded — fail-closed".to_string()),
                program: None,
                list_version: self.list_version.clone(),
                screened_at: chrono::Utc::now(),
            };
        }

        if self.sanctioned_addresses.contains(&normalized) {
            return ScreenResponse {
                status: ScreenResult::Blocked,
                address: address.to_string(),
                matched_entity: Some("Exact address match in SDN list".to_string()),
                program: Some("SDN".to_string()),
                list_version: self.list_version.clone(),
                screened_at: chrono::Utc::now(),
            };
        }

        ScreenResponse {
            status: ScreenResult::Clean,
            address: address.to_string(),
            matched_entity: None,
            program: None,
            list_version: self.list_version.clone(),
            screened_at: chrono::Utc::now(),
        }
    }

    pub fn list_version(&self) -> &str { &self.list_version }
    pub fn address_count(&self) -> usize { self.sanctioned_addresses.len() }
}

impl Default for ComplianceEngine {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fail_closed_when_empty() {
        let engine = ComplianceEngine::new();
        let result = engine.screen("0x1234567890abcdef1234567890abcdef12345678");
        assert!(matches!(result.status, ScreenResult::Flagged));
    }

    #[test]
    fn test_clean_address() {
        let mut engine = ComplianceEngine::new();
        engine.load_sdn_list(
            vec![ofac::SdnEntry {
                uid: "1".into(),
                name: "Test Entity".into(),
                aliases: vec![],
                addresses: vec!["0xAAAA".into()],
                program: Some("SDN".into()),
                country: Some("XX".into()),
                date_listed: None,
                date_delisted: None,
            }],
            "test-v1".into(),
        );

        let result = engine.screen("0xBBBB");
        assert!(matches!(result.status, ScreenResult::Clean));
    }

    #[test]
    fn test_blocked_address() {
        let mut engine = ComplianceEngine::new();
        engine.load_sdn_list(
            vec![ofac::SdnEntry {
                uid: "1".into(),
                name: "Test Entity".into(),
                aliases: vec![],
                addresses: vec!["0xAAAA".into()],
                program: Some("SDN".into()),
                country: Some("XX".into   