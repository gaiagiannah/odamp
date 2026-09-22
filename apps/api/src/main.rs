//! ODAMP API — Axum server.
//!
//! Router wiring only. All logic lives in routes/ modules.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod db;
mod state;
mod middleware;
mod routes;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Init logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "odamp_api=debug,tower_http=debug".into()),
        )
        .init();

    // Load config
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://odamp:odamp_dev@localhost:5432/odamp".into());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev_secret_change_me_min_32_chars_long".into());
    let port: u16 = std::env::var("API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    // Connect to DB
    let pool = db::create_pool(&database_url).await?;

    // Load OFAC SDN list into memory
    let mut compliance = odamp_compliance::ComplianceEngine::new();
    // In production, this loads from the DB (populated by load-ofac-sdn.sh)
    // For now, start empty (fail-closed: everything is FLAGGED until loaded)
    tracing::info!("Compliance engine initialized (SDN list will be loaded from DB)");

    // Build state
    let state = AppState {
        pool,
        jwt_secret,
        compliance: Arc::new(std::sync::Mutex::new(compliance)),
    };

    // Build router
    let app = Router::new()
        .route("/health", get(routes::health::health))
        .route("/wallets", post(routes::wallet::create_wallet))
        .route("/wallets/:id", get(routes::wallet::get_wallet))
        .route("/wallets/:id/sign", post(routes::sign::sign))
        .route("/positions", get(routes::positions::get_portfolio))
        .route("/positions/sync", post(routes::sync::sync))
        .route("/compliance/screen", post(routes::compliance::screen))
        .route("/agents/risk-sentinel/analyze", post(routes::agents::analyze))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("ODAMP API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}   