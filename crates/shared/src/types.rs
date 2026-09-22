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
    pub encrypted_share: String, // base64-encoded AES-256-GCM ciphertext
}

#[derive(Debug, Deserialize)]
pub struct SignRequest {
    pub wallet_id: Uuid,
    pub message: String, // hex-encoded bytes to sign
    pub shares: Vec<EncryptedShare>,
}

#[derive(Debug, Deserialize)]
pub struct EncryptedShare {
    pub share_id: u32,
    pub encrypted_share: String,
    pub password: String, // decrypted client-side, never stored
}

#[derive(Debug, Serialize)]
pub struct SignResponse {
    pub signature: String, // 65-byte hex (R || z)
    pub wallet_id: Uuid,
    pub signed_at: DateTime<Utc>,
}

// ─── Portfolio ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub chain: String,
    pub token_address: Option<String>, // None for native
    pub symbol: String,
    pub balance: String,               // human-readable (decimal string)
    pub price_usd: f64,
    pub value_usd: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PortfolioSummary {
    pub wallet_id: Uuid,
    pub total_value_usd: f64,
    pub positions: Vec<Position>,
    pub risk: RiskMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub var_95: f64,
    pub max_drawdown: f64,
    pub hhi_concentration: f64,
    pub volatility_30d: f64,
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

// ─── Execution ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct QuoteRequest {
    pub wallet_id: Uuid,
    pub chain: String,
    pub from_token: String, // address or "NATIVE"
    pub to_token: String,
    pub amount: String,     // human-readable
    pub slippage_bps: u32,  // basis points (50 = 0.5%)
}

#[derive(Debug, Serialize)]
pub struct QuoteResponse {
    pub quote_id: Uuid,
    pub from_token: String,
    pub to_token: String,
    pub amount_in: String,
    pub expected_out: String,
    pub slippage_bps: u32,
    pub gas_estimate: u64,
    pub gas_cost_usd: f64,
    pub route: Vec<String>,
    pub quoted_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    pub quote_id: Uuid,
    pub shares: Vec<EncryptedShare>,
}

#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    pub tx_hash: String,
    pub status: String, // "pending" | "confirmed" | "failed"
    pub block: Option<u64>,
    pub executed_at: DateTime<Utc>,
}   