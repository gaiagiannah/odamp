
//! ODAMP Compliance Core
//!
//! Sanctions screening against OFAC's Specially Designated Nationals (SDN) list.
//! Public data, no API key required. Fail-closed: if list not loaded, FLAG.

pub mod ofac;

use odamp_shared::{ScreenResponse, ScreenResult};
use std::collections::HashSet;

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
                country: Some("XX".into()),
                date_listed: None,
                date_delisted: None,
            }],
            "test-v1".into(),
        );
        let result = engine.screen("0xaaaa");
        assert!(matches!(result.status, ScreenResult::Blocked));
    }
}
EOF

cat > crates/compliance/src/ofac.rs << 'EOF'
//! OFAC SDN feed parsing and fetching.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdnEntry {
    pub uid: String,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub addresses: Vec<String>,
    #[serde(default)]
    pub program: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub date_listed: Option<String>,
    #[serde(default)]
    pub date_delisted: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SdnListResponse {
    #[serde(rename = "results")]
    pub entries: Vec<SdnEntry>,
    #[serde(default)]
    pub count: usize,
}

pub fn parse_sdn_response(json: &str) -> Result<Vec<SdnEntry>, serde_json::Error> {
    let response: SdnListResponse = serde_json::from_str(json)?;
    Ok(response.entries)
}

pub fn list_version(entries: &[SdnEntry]) -> String {
    let addr_count: usize = entries.iter().map(|e| e.addresses.len()).sum();
    format!(
        "{} entries, {} addresses, {}",
        entries.len(),
        addr_count,
        chrono::Utc::now().to_rfc3339()
    )
}

pub async fn fetch_sdn_list(url: &str) -> Result<Vec<SdnEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let response = reqwest::get(url).await?.text().await?;
    Ok(parse_sdn_response(&response)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sdn_json() {
        let json = r#"{
            "results": [
                {
                    "uid": "123",
                    "name": "Test Person",
                    "aliases": ["Alias One"],
                    "addresses": ["0xABC", "bc1qxyz"],
                    "program": "SDN",
                    "country": "RU"
                }
            ],
            "count": 1
        }"#;

        let entries = parse_sdn_response(json).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Test Person");
        assert_eq!(entries[0].addresses.len(), 2);
    }
}
EOF

# ─── crates/portfolio ──────────────────────────────────────────
cat > crates/portfolio/Cargo.toml << 'EOF'
[package]
name = "odamp-portfolio"
version.workspace = true
edition.workspace = true

[dependencies]
odamp-shared.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
thiserror.workspace = true
EOF

cat > crates/portfolio/src/lib.rs << 'EOF'
//! ODAMP Portfolio Engine
//!
//! Position tracking and risk-adjusted analytics.
//! All math is computed from input data — no stubs.

pub mod position;
pub mod risk;

pub use position::Position;
pub use risk::{compute_risk_metrics, RiskMetrics, PriceHistory};
EOF

cat > crates/portfolio/src/position.rs << 'EOF'
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

impl Position {
    pub fn new(
        wallet_id: Uuid,
        chain: &str,
        token_address: Option<String>,
        symbol: &str,
        balance: &str,
        price_usd: f64,
    ) -> Self {
        let balance_f64: f64 = balance.parse().unwrap_or(0.0);
        Self {
            id: Uuid::new_v4(),
            wallet_id,
            chain: chain.to_string(),
            token_address,
            symbol: symbol.to_string(),
            balance: balance.to_string(),
            price_usd,
            value_usd: balance_f64 * price_usd,
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_value_calculation() {
        let wallet_id = Uuid::new_v4();
        let pos = Position::new(wallet_id, "ethereum", None, "ETH", "4.0", 3500.0);
        assert!((pos.value_usd - 14000.0).abs() < 0.01);
    }
}
EOF

cat > crates/portfolio/src/risk.rs << 'EOF'
//! Risk metrics: Sharpe, Sortino, VaR, max drawdown, HHI, volatility.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub var_95: f64,
    pub max_drawdown: f64,
    pub hhi_concentration: f64,
    pub volatility_30d: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistory {
    pub symbol: String,
    pub prices: Vec<f64>,
    pub weights: Vec<f64>,
}

pub fn compute_risk_metrics(
    histories: &[PriceHistory],
    positions_value: &[f64],
    risk_free_rate: f64,
) -> RiskMetrics {
    let total_value: f64 = positions_value.iter().sum();

    let hhi = if total_value > 0.0 {
        positions_value.iter().map(|v| {
            let share = v / total_value;
            share * share
        }).sum()
    } else {
        0.0
    };

    if histories.is_empty() || histories[0].prices.len() < 2 {
        return RiskMetrics {
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            var_95: 0.0,
            max_drawdown: 0.0,
            hhi_concentration: hhi,
            volatility_30d: 0.0,
        };
    }

    let n_periods = histories[0].prices.len() - 1;
    let mut portfolio_returns: Vec<f64> = vec![0.0; n_periods];

    for i in 0..n_periods {
        for h in histories {
            let weight = h.weights.first().copied().unwrap_or(1.0 / histories.len() as f64);
            let prev = h.prices[i];
            let curr = h.prices[i + 1];
            if prev > 0.0 {
                portfolio_returns[i] += ((curr - prev) / prev) * weight;
            }
        }
    }

    let annualization = 252.0_f64.sqrt();
    let mean_return = portfolio_returns.iter().sum::<f64>() / n_periods as f64;

    let variance = portfolio_returns.iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>() / n_periods as f64;
    let volatility = variance.sqrt();
    let volatility_30d = volatility * annualization;

    let sharpe = if volatility > 0.0 {
        (mean_return * 252.0 - risk_free_rate) / (volatility * annualization)
    } else {
        0.0
    };

    let downside_sum: f64 = portfolio_returns.iter()
        .filter(|r| **r < 0.0)
        .map(|r| r.powi(2))
        .sum();
    let downside_deviation = (downside_sum / n_periods as f64).sqrt();
    let sortino = if downside_deviation > 0.0 {
        (mean_return * 252.0 - risk_free_rate) / (downside_deviation * annualization)
    } else {
        0.0
    };

    let mut sorted_returns = portfolio_returns.clone();
    sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let var_index = (n_periods as f64 * 0.05) as usize;
    let var_95 = -sorted_returns.get(var_index).copied().unwrap_or(0.0);

    let mut peak = f64::MIN;
    let mut max_dd = 0.0;
    let mut cumulative = 1.0;
    for r in &portfolio_returns {
        cumulative *= 1.0 + r;
        peak = peak.max(cumulative);
        if peak > 0.0 {
            max_dd = max_dd.max((peak - cumulative) / peak);
        }
    }

    RiskMetrics {
        sharpe_ratio: sharpe,
        sortino_ratio: sortino,
        var_95,
        max_drawdown: max_dd,
        hhi_concentration: hhi,
        volatility_30d,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hhi_single_asset() {
        let metrics = compute_risk_metrics(&[], &[10000.0], 0.0);
        assert!((metrics.hhi_concentration - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_hhi_equal_split() {
        let metrics = compute_risk_metrics(&[], &[5000.0, 5000.0], 0.0);
        assert!((metrics.hhi_concentration - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_sharpe_positive_returns() {
        let history = PriceHistory {
            symbol: "ETH".into(),
            prices: vec![100.0, 101.0, 102.0, 103.0, 104.0],
            weights: vec![1.0],
        };
        let metrics = compute_risk_metrics(&[history], &[1000.0], 0.0);
        assert!(metrics.sharpe_ratio > 0.0);
        assert!(metrics.max_drawdown < 0.01);
    }
}
EOF

# ─── crates/indexer ────────────────────────────────────────────
cat > crates/indexer/Cargo.toml << 'EOF'
[package]
name = "odamp-indexer"
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
hex.workspace = true
EOF

cat > crates/indexer/src/lib.rs << 'EOF'
//! ODAMP On-Chain Indexer
//!
//! Real data sources, no mocking:
//! 1. EvmClient — reads balances from EVM JSON-RPC
//! 2. PriceClient — reads USD prices from CoinGecko

pub mod evm;
pub mod pricing;

pub use evm::EvmClient;
pub use pricing::PriceClient;
EOF

cat > crates/indexer/src/evm.rs << 'EOF'
//! EVM chain client: native + ERC-20 balance reads.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EvmError {
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Decoding error: {0}")]
    Decode(String),
}

pub struct EvmClient {
    rpc_url: String,
    http: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: T,
    #[serde(default)]
    error: Option<serde_json::Value>,
}

impl EvmClient {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Get native token balance in wei.
    pub async fn get_native_balance(&self, address: &str) -> Result<String, EvmError> {
        let req = RpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "eth_getBalance".into(),
            params: vec![
                serde_json::json!(address),
                serde_json::json!("latest"),
            ],
        };

        let resp: RpcResponse<String> = self
            .http
            .post(&self.rpc_url)
            .json(&req)
            .send()
            .await
            .map_err(|e| EvmError::Rpc(e.to_string()))?
            .json()
            .await
            .map_err(|e| EvmError::Rpc(e.to_string()))?;

        if let Some(err) = resp.error {
            return Err(EvmError::Rpc(err.to_string()));
        }

        let hex_str = resp.result.trim_start_matches("0x");
        let wei = u128::from_str_radix(hex_str, 16)
            .map_err(|e| EvmError::Decode(e.to_string()))?;
        Ok(wei.to_string())
    }

    /// Get ERC-20 token balance (raw units).
    pub async fn get_erc20_balance(&self, token_address: &str, wallet_address: &str) -> Result<String, EvmError> {
        let addr_hex = wallet_address.trim_start_matches("0x");
        let padded = format!("{:064}", addr_hex);
        let data = format!("0x70a08231{}", padded);

        let req = RpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "eth_call".into(),
            params: vec![
                serde_json::json!({ "to": token_address, "data": data }),
                serde_json::json!("latest"),
            ],
        };

        let resp: RpcResponse<String> = self
            .http
            .post(&self.rpc_url)
            .json(&req)
            .send()
            .await
            .map_err(|e| EvmError::Rpc(e.to_string()))?
            .json()
            .await
            .map_err(|e| EvmError::Rpc(e.to_string()))?;

        if let Some(err) = resp.error {
            return Err(EvmError::Rpc(err.to_string()));
        }

        let hex_str = resp.result.trim_start_matches("0x");
        if hex_str.is_empty() || hex_str == "0" {
            return Ok("0".to_string());
        }
        let value = u128::from_str_radix(hex_str, 16)
            .map_err(|e| EvmError::Decode(e.to_string()))?;
        Ok(value.to_string())
    }
}
EOF

cat > crates/indexer/src/pricing.rs << 'EOF'
//! Price client: CoinGecko public API.

use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriceError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Token not found: {0}")]
    TokenNotFound(String),
}

pub struct PriceClient {
    base_url: String,
    api_key: Option<String>,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct PriceEntry {
    usd: f64,
}

impl PriceClient {
    pub fn new(base_url: &str, api_key: Option<String>) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            http: reqwest::Client::new(),
        }
    }

    pub async fn get_prices(&self, coingecko_ids: &[&str]) -> Result<HashMap<String, f64>, PriceError> {
        let ids = coingecko_ids.join(",");
        let url = format!("{}/simple/price?ids={}&vs_currencies=usd", self.base_url, ids);

        let mut req = self.http.get(&url);
        if let Some(key) = &self.api_key {
            req = req.header("x-cg-demo-api-key", key);
        }

        let resp: HashMap<String, PriceEntry> = req
            .send()
            .await
            .map_err(|e| PriceError::Api(e.to_string()))?
            .json()
            .await
            .map_err(|e| PriceError::Api(e.to_string()))?;

        Ok(resp.into_iter().map(|(k, v)| (k, v.usd)).collect())
    }

    pub async fn get_price(&self, coingecko_id: &str) -> Result<f64, PriceError> {
        let prices = self.get_prices(&[coingecko_id]).await?;
        prices.get(coingecko_id)
            .copied()
            .ok_or_else(|| PriceError::TokenNotFound(coingecko_id.to_string()))
    }
}
EOF

# ─── apps/api ──────────────────────────────────────────────────
cat > apps/api/Cargo.toml << 'EOF'
[package]
name = "odamp-api"
version.workspace = true
edition.workspace = true

[[bin]]
name = "odamp-api"
path = "src/main.rs"

[dependencies]
odamp-shared.workspace = true
odamp-security-core.workspace = true
odamp-compliance.workspace = true
odamp-portfolio.workspace = true
odamp-indexer.workspace = true
axum.workspace = true
tokio.workspace = true
tower-http.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
sqlx.workspace = true
jsonwebtoken.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
thiserror.workspace = true
anyhow.workspace = true
reqwest.workspace = true
EOF

cat > apps/api/src/main.rs << 'EOF'
use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod db;
mod state;
mod routes;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "odamp_api=debug,tower_http=debug".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://odamp:odamp_dev@localhost:5432/odamp".into());
    let port: u16 = std::env::var("API_PORT")
        .ok().and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let pool = db::create_pool(&database_url).await?;

    let compliance = odamp_compliance::ComplianceEngine::new();

    let state = AppState {
        pool,
        compliance: Arc::new(std::sync::Mutex::new(compliance)),
    };

    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/wallets", post(routes::create_wallet))
        .route("/positions", get(routes::get_portfolio))
        .route("/positions/sync", post(routes::sync))
        .route("/compliance/screen", post(routes::screen))
        .route("/agents/risk-sentinel/analyze", post(routes::analyze))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("ODAMP API listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
EOF

cat > apps/api/src/state.rs << 'EOF'
use std::sync::Arc;
use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
    pub compliance: Arc<std::sync::Mutex<odamp_compliance::ComplianceEngine>>,
}
EOF

cat > apps/api/src/db.rs << 'EOF'
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .connect(database_url)
        .await?;
    tracing::info!("Database pool connected");
    Ok(pool)
}
EOF

cat > apps/api/src/routes.rs << 'EOF'
use axum::extract::State;
use axum::Json;
use crate::state::AppState;

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub async fn create_wallet(
    State(state): State<AppState>,
    Json(req): Json<odamp_shared::CreateWalletRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let password = std::env::var("KEY_ENCRYPTION_PASSWORD")
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "KEY_ENCRYPTION_PASSWORD not set".into()))?;

    let result = odamp_security_core::generate_key_shares(
        req.total_shares,
        req.threshold,
        &password,
    )
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "wallet": {
            "group_public_key": result.group_public_key,
            "threshold": result.threshold,
            "total_shares": result.total_shares,
            "chain": req.chain,
        },
        "key_shares": result.shares,
        "warning": "Store these key shares securely. They will NOT be available again."
    })))
}

pub async fn get_portfolio(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // TODO: accept wallet_id query param, query positions table
    Ok(Json(serde_json::json!({
        "positions": [],
        "total_value_usd": 0.0,
        "note": "Wire up DB query with wallet_id filter"
    })))
}

pub async fn sync(
    State(_state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    // TODO: parse SyncRequest, call EvmClient + PriceClient, upsert positions
    Json(serde_json::json!({
        "status": "accepted",
        "targets": req.get("targets").and_then(|t| t.as_array()).map(|a| a.len()).unwrap_or(0),
        "note": "Wire up EvmClient + PriceClient calls"
    }))
}

pub async fn screen(
    State(state): State<AppState>,
    Json(req): Json<odamp_shared::ScreenRequest>,
) -> Json<odamp_shared::ScreenResponse> {
    let compliance = state.compliance.lock().unwrap();
    Json(compliance.screen(&req.address))
}

pub async fn analyze(
    State(_state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let ai_url = std::env::var("AI_AGENT_URL")
        .unwrap_or_else(|_| "http://localhost:8000".into());

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/analyze", ai_url))
        .json(&req)
        .send()
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, format!("AI agent unreachable: {}", e)))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))?;

    Ok(Json(response))
}
EOF

# ─── apps/cli ──────────────────────────────────────────────────
cat > apps/cli/Cargo.toml << 'EOF'
[package]
name = "odamp-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "odamp"
path = "src/main.rs"

[dependencies]
odamp-shared.workspace = true
odamp-security-core.workspace = true
odamp-compliance.workspace = true
odamp-indexer.workspace = true
clap.workspace = true
tokio.workspace = true
serde_json.workspace = true
uuid.workspace = true
colored.workspace = true
anyhow.workspace = true
hex.workspace = true
base64.workspace = true
EOF

cat > apps/cli/src/main.rs << 'EOF'
use clap::{Parser, Subcommand};
use colored::Colorized;

#[derive(Parser)]
#[command(name = "odamp", version, about = "ODAMP CLI — Digital Asset Management")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Wallet operations
    Wallet {
        #[command(subcommand)]
        action: WalletCommands,
    },
    /// Sign a message (FROST threshold)
    Sign {
        #[arg(long)]
        message: String,
        #[arg(long)]
        shares: String,
    },
    /// Check on-chain balance
    Balance {
        #[arg(long)]
        address: String,
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
    /// Screen address against OFAC SDN
    Screen {
        #[arg(long)]
        address: String,
    },
}

#[derive(Subcommand)]
enum WalletCommands {
    /// Create threshold wallet (DKG)
    Create {
        #[arg(long, default_value_t = 3)]
        shares: u32,
        #[arg(long, default_value_t = 2)]
        threshold: u32,
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
    /// List stored key shares
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Wallet { action } => match action {
            WalletCommands::Create { shares, threshold, chain } => {
                let password = std::env::var("KEY_ENCRYPTION_PASSWORD")
                    .map_err(|_| anyhow::anyhow!("Set KEY_ENCRYPTION_PASSWORD in your environment"))?;

                println!("{}", "ODAMP — Threshold Wallet Creation".bold());
                println!("{}", format!("  Shares: {}  Threshold: {}  Chain: {}", shares, threshold, chain).dimmed());
                println!("{}", "Running DKG...".yellow());

                let result = odamp_security_core::generate_key_shares(shares, threshold, &password)?;

                let store = odamp_security_core::KeyStore::default_location()?;
                for share in &result.shares {
                    let file = odamp_security_core::EncryptedShareFile {
                        share_id: share.share_id,
                        ciphertext: share.encrypted_share.clone(),
                        salt: String::new(),
                        created_at: chrono::Utc::now().to_rfc3339(),
                    };
                    store.save_share(&file)?;
                }

                println!("{}", "✓ Wallet created".green().bold());
                println!("{}", format!("  Group Public Key: {}", result.group_public_key).bold());
                println!("{}", format!("  Shares in: ~/.odamp/keys/").dimmed());
                println!("{}", format!("  Need {} of {} shares to sign", threshold, shares).yellow());
            }
            WalletCommands::List => {
                let store = odamp_security_core::KeyStore::default_location()?;
                let shares = store.list_shares();
                if shares.is_empty() {
                    println!("{}", "No key shares found in ~/.odamp/keys/".yellow());
                } else {
                    println!("{}", "Stored shares:".bold());
                    for id in shares { println!("  share_{}.json", id); }
                }
            }
        },
        Commands::Sign { message, shares } => {
            let password = std::env::var("KEY_ENCRYPTION_PASSWORD")
                .map_err(|_| anyhow::anyhow!("Set KEY_ENCRYPTION_PASSWORD"))?;
            let msg_bytes = hex::decode(&message)?;
            let share_ids: Vec<u32> = shares.split(',').map(|s| s.trim().parse()).collect::<Result<Vec<_>, _>>()?;

            println!("{}", "ODAMP — FROST Signing".bold());
            println!("{}", format!("  Shares: {:?}  Msg: {} bytes", share_ids, msg_bytes.len()).dimmed());

            let store = odamp_security_core::KeyStore::default_location()?;
            let mut enc_shares: Vec<(u32, Vec<u8>)> = Vec::new();
            let mut passwords: std::collections::HashMap<u32, String> = std::collections::HashMap::new();

            for id in &share_ids {
                let file = store.load_share(*id)?;
                let dec = base64::decode(&file.ciphertext)?;
                enc_shares.push((*id, dec));
                passwords.insert(*id, password.clone());
            }

            match odamp_security_core::sign_message(
                &msg_bytes,
                &enc_shares.iter().map(|(id, b)| (*id, b.as_slice())).collect::<Vec<_>>(),
                &passwords.iter().map(|(k, v)| (*k, v.as_str())).collect::<std::collections::HashMap<u32, &str>>(),
                2,
            ) {
                Ok(result) => {
                    println!("{}", "✓ Signature:   