//! ODAMP API — Phase 1: real Postgres persistence + non-custodial signing
//!
//! Changes from Phase 0:
//! - Wallet creation persists only PUBLIC wallet info to Postgres
//!   (group public key, threshold config). Key shares are returned
//!   to the caller once and never stored server-side, anywhere —
//!   see core/security/src/lib.rs for why this matters.
//! - Signing now requires the caller to supply the shares needed to
//!   meet the wallet's threshold, in the request body. This is more
//!   awkward to `curl` by hand than Phase 0's version, and that
//!   awkwardness is the point: it's what "the server can't move your
//!   funds alone" actually looks like at the API level.
//! - Positions are persisted per-user instead of being pure
//!   stateless compute-and-forget.
//! - Sanctions screening now writes an audit-trail row to
//!   `transactions` for every screened transfer.
//!
//! Requires a running Postgres (see ../docker-compose.yml) and a
//! DATABASE_URL env var (see ../.env.example).

mod db;

use axum::{
    extract::{Json, Path, State},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use odamp_compliance::SanctionsList;
use odamp_portfolio::{Portfolio, Position};
use odamp_security::{generate_wallet_trusted_dealer, sign_threshold, ExportedShare};

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    sanctions: std::sync::Arc<SanctionsList>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok(); // loads .env if present; fine if it's absent (e.g. in CI)

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set — see .env.example, and make sure docker-compose's postgres is running");

    let pool = db::connect(&database_url).await?;
    db::run_migrations(&pool).await?;
    tracing::info!("connected to Postgres and ran migrations");

    // Placeholder sanctions data for local dev — see README, Phase 3
    // replaces this with a real scheduled OFAC SDN feed.
    let sanctions = SanctionsList::load_from_json(
        r#"["0x0000000000000000000000000000000000dEaD"]"#,
        "local-dev-placeholder — NOT a live OFAC feed",
    )
    .expect("static placeholder list must parse");

    let state = AppState {
        pool,
        sanctions: std::sync::Arc::new(sanctions),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/users", post(create_user))
        .route("/api/v1/security/wallet", post(create_wallet))
        .route("/api/v1/security/wallet/sign", post(sign_message))
        .route("/api/v1/portfolio/:user_id/positions", post(add_position))
        .route("/api/v1/portfolio/:user_id", get(get_portfolio))
        .route("/api/v1/compliance/screen", post(screen_address))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("ODAMP API listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

// ---------- Users ----------

#[derive(Deserialize)]
struct CreateUserRequest {
    jurisdiction: String,
}

#[derive(Serialize)]
struct CreateUserResponse {
    user_id: Uuid,
}

async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, (axum::http::StatusCode, String)> {
    let user_id = db::create_user(&state.pool, &req.jurisdiction)
        .await
        .map_err(internal_error)?;
    Ok(Json(CreateUserResponse { user_id }))
}

// ---------- Security / wallets ----------

#[derive(Deserialize)]
struct CreateWalletRequest {
    user_id: Uuid,
    threshold: u16,
    total_shares: u16,
}

#[derive(Serialize)]
struct CreateWalletResponse {
    wallet_id: Uuid,
    group_public_key_hex: String,
    /// Returned exactly once. Distribute each of these to a
    /// DIFFERENT custody domain (this device, an encrypted backup,
    /// an HSM). The server does not keep a copy after this response
    /// is sent — if you lose them all, the wallet is unrecoverable
    /// in this Phase 1 scaffold (social recovery is a later phase).
    shares: Vec<ExportedShare>,
}

async fn create_wallet(
    State(state): State<AppState>,
    Json(req): Json<CreateWalletRequest>,
) -> Result<Json<CreateWalletResponse>, (axum::http::StatusCode, String)> {
    if !db::user_exists(&state.pool, req.user_id)
        .await
        .map_err(internal_error)?
    {
        return Err((axum::http::StatusCode::NOT_FOUND, "user not found".into()));
    }

    let generated = generate_wallet_trusted_dealer(req.threshold, req.total_shares)
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))?;

    let wallet_id = db::insert_wallet(
        &state.pool,
        req.user_id,
        generated.info.threshold,
        generated.info.total_shares,
        &generated.info.group_public_key_hex,
    )
    .await
    .map_err(internal_error)?;

    Ok(Json(CreateWalletResponse {
        wallet_id,
        group_public_key_hex: generated.info.group_public_key_hex,
        shares: generated.shares,
    }))
}

#[derive(Deserialize)]
struct SignRequest {
    wallet_id: Uuid,
    message: String,
    /// At least `threshold` of these must be supplied — see the
    /// wallet's `threshold` field (fetched server-side from
    /// Postgres, not trusted from the client).
    shares: Vec<ExportedShare>,
}

#[derive(Serialize)]
struct SignResponse {
    signature_hex: String,
}

async fn sign_message(
    State(state): State<AppState>,
    Json(req): Json<SignRequest>,
) -> Result<Json<SignResponse>, (axum::http::StatusCode, String)> {
    let wallet = db::get_wallet(&state.pool, req.wallet_id)
        .await
        .map_err(internal_error)?
        .ok_or((axum::http::StatusCode::NOT_FOUND, "wallet not found".into()))?;

    let signature = sign_threshold(
        wallet.threshold as u16,
        wallet.total_shares as u16,
        req.message.as_bytes(),
        &req.shares,
    )
    .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))?;

    let sig_bytes = signature
        .serialize()
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(SignResponse {
        signature_hex: hex::encode(sig_bytes),
    }))
}

// ---------- Portfolio ----------

async fn add_position(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(position): Json<Position>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
    db::upsert_position(&state.pool, user_id, &position)
        .await
        .map_err(internal_error)?;
    Ok(axum::http::StatusCode::CREATED)
}

#[derive(Serialize)]
struct PortfolioResponse {
    total_value_usd: f64,
    total_unrealized_pnl_usd: f64,
    allocation_by_type: std::collections::BTreeMap<String, f64>,
    position_count: usize,
}

async fn get_portfolio(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<PortfolioResponse>, (axum::http::StatusCode, String)> {
    let positions = db::get_positions_for_user(&state.pool, user_id)
        .await
        .map_err(internal_error)?;
    let position_count = positions.len();

    let portfolio = Portfolio { user_id, positions };

    Ok(Json(PortfolioResponse {
        total_value_usd: portfolio.total_value_usd(),
        total_unrealized_pnl_usd: portfolio.total_unrealized_pnl_usd(),
        allocation_by_type: portfolio.allocation_by_type(),
        position_count,
    }))
}

// ---------- Compliance ----------

#[derive(Deserialize)]
struct ScreenRequest {
    user_id: Uuid,
    wallet_id: Option<Uuid>,
    chain: String,
    from_address: String,
    to_address: String,
}

#[derive(Serialize)]
struct ScreenResponse {
    blocked: bool,
    results: Vec<odamp_compliance::ScreeningResult>,
    transaction_id: Uuid,
}

async fn screen_address(
    State(state): State<AppState>,
    Json(req): Json<ScreenRequest>,
) -> Result<Json<ScreenResponse>, (axum::http::StatusCode, String)> {
    let (blocked, results) = state
        .sanctions
        .screen_transaction(&req.from_address, &req.to_address);

    let transaction_id = db::record_screened_transaction(
        &state.pool,
        req.user_id,
        req.wallet_id,
        &req.chain,
        &req.from_address,
        &req.to_address,
        blocked,
    )
    .await
    .map_err(internal_error)?;

    Ok(Json(ScreenResponse {
        blocked,
        results,
        transaction_id,
    }))
}

fn internal_error(e: anyhow::Error) -> (axum::http::StatusCode, String) {
    tracing::error!("internal error: {e:?}");
    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
