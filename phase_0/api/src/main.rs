//! ODAMP API — Phase 0 skeleton
//!
//! This is intentionally small: a health check, a threshold-wallet
//! creation endpoint that calls the real FROST implementation in
//! `odamp-security`, a portfolio-analytics endpoint that calls the
//! real math in `odamp-portfolio`, and a sanctions-screening
//! endpoint that calls `odamp-compliance`. There is no database yet
//! (see /migrations for the schema that Phase 0's follow-up wires
//! up) — state here is in-memory and resets on restart. That's
//! correct for this phase: prove each core module works end-to-end
//! through a real HTTP path before adding persistence.

use std::sync::{Arc, Mutex};

use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

use odamp_compliance::SanctionsList;
use odamp_portfolio::{Portfolio, Position};
use odamp_security::{generate_wallet_trusted_dealer, sign_threshold, ThresholdWallet};

#[derive(Clone)]
struct AppState {
    sanctions: Arc<SanctionsList>,
    // Demo in-memory wallet store, Phase 0 only. Real key material
    // belongs in per-user encrypted storage, not a shared process
    // Mutex — this exists purely so the /wallet/sign endpoint has
    // something to sign against for now.
    wallets: Arc<Mutex<Vec<ThresholdWallet>>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Placeholder sanctions data for local dev. Replace with a real
    // scheduled fetch of OFAC's published SDN digital-currency
    // address list before this touches real transactions.
    let sanctions = SanctionsList::load_from_json(
        r#"["0x0000000000000000000000000000000000dEaD"]"#,
        "local-dev-placeholder — NOT a live OFAC feed",
    )
    .expect("static placeholder list must parse");

    let state = AppState {
        sanctions: Arc::new(sanctions),
        wallets: Arc::new(Mutex::new(Vec::new())),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/security/wallet", post(create_wallet))
        .route("/api/v1/security/wallet/sign", post(sign_message))
        .route("/api/v1/portfolio/analyze", post(analyze_portfolio))
        .route("/api/v1/compliance/screen", post(screen_address))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind port 8080");

    tracing::info!("ODAMP API listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Deserialize)]
struct CreateWalletRequest {
    threshold: u16,
    total_shares: u16,
}

#[derive(Serialize)]
struct CreateWalletResponse {
    wallet_id: String,
    threshold: u16,
    total_shares: u16,
    group_public_key_hex: String,
}

async fn create_wallet(
    State(state): State<AppState>,
    Json(req): Json<CreateWalletRequest>,
) -> Result<Json<CreateWalletResponse>, (axum::http::StatusCode, String)> {
    let wallet = generate_wallet_trusted_dealer(req.threshold, req.total_shares)
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))?;

    let response = CreateWalletResponse {
        wallet_id: wallet.wallet_id.to_string(),
        threshold: wallet.threshold,
        total_shares: wallet.total_shares,
        group_public_key_hex: hex::encode(
            wallet.public_key_package.verifying_key().serialize(),
        ),
    };

    state.wallets.lock().unwrap().push(wallet);
    Ok(Json(response))
}

#[derive(Deserialize)]
struct SignRequest {
    wallet_id: String,
    message: String,
}

#[derive(Serialize)]
struct SignResponse {
    signature_hex: String,
}

async fn sign_message(
    State(state): State<AppState>,
    Json(req): Json<SignRequest>,
) -> Result<Json<SignResponse>, (axum::http::StatusCode, String)> {
    let wallets = state.wallets.lock().unwrap();
    let wallet = wallets
        .iter()
        .find(|w| w.wallet_id.to_string() == req.wallet_id)
        .ok_or((axum::http::StatusCode::NOT_FOUND, "wallet not found".into()))?;

    let signer_ids: Vec<_> = wallet
        .shares
        .keys()
        .take(wallet.threshold as usize)
        .copied()
        .collect();

    let signature = sign_threshold(wallet, req.message.as_bytes(), &signer_ids)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(SignResponse {
        signature_hex: hex::encode(signature.serialize().unwrap_or_default()),
    }))
}

#[derive(Deserialize)]
struct AnalyzePortfolioRequest {
    positions: Vec<Position>,
}

#[derive(Serialize)]
struct AnalyzePortfolioResponse {
    total_value_usd: f64,
    total_unrealized_pnl_usd: f64,
    allocation_by_type: std::collections::BTreeMap<String, f64>,
}

async fn analyze_portfolio(
    Json(req): Json<AnalyzePortfolioRequest>,
) -> Json<AnalyzePortfolioResponse> {
    let portfolio = Portfolio {
        user_id: uuid::Uuid::new_v4(),
        positions: req.positions,
    };

    Json(AnalyzePortfolioResponse {
        total_value_usd: portfolio.total_value_usd(),
        total_unrealized_pnl_usd: portfolio.total_unrealized_pnl_usd(),
        allocation_by_type: portfolio.allocation_by_type(),
    })
}

#[derive(Deserialize)]
struct ScreenRequest {
    from_address: String,
    to_address: String,
}

#[derive(Serialize)]
struct ScreenResponse {
    blocked: bool,
    results: Vec<odamp_compliance::ScreeningResult>,
}

async fn screen_address(
    State(state): State<AppState>,
    Json(req): Json<ScreenRequest>,
) -> Json<ScreenResponse> {
    let (blocked, results) = state
        .sanctions
        .screen_transaction(&req.from_address, &req.to_address);
    Json(ScreenResponse { blocked, results })
}
