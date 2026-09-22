use axum::extract::{Path, State};
use axum::Json;
use odamp_shared::{CreateWalletRequest, CreateWalletResponse};
use crate::state::AppState;

pub async fn create_wallet(
    State(state): State<AppState>,
    Json(req): Json<CreateWalletRequest>,
) -> Result<Json<CreateWalletResponse>, (axum::http::StatusCode, String)> {
    // Run DKG
    let password = std::env::var("KEY_ENCRYPTION_PASSWORD")
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "KEY_ENCRYPTION_PASSWORD not set".into()))?;

    let result = odamp_security_core::generate_key_shares(
        req.total_shares,
        req.threshold,
        &password,
    )
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Persist public wallet info to DB
    // (key shares are returned to caller and NEVER stored server-side)

    Ok(Json(CreateWalletResponse {
        wallet: odamp_shared::WalletInfo {
            id: uuid::Uuid::new_v4(),
            user_id: req.user_id,
            chain: req.chain,
            address: result.group_public_key.clone(), // In production, derive Safe address from group pubkey
            group_public_key: result.group_public_key,
            threshold: result.threshold,
            total_shares: result.total_shares,
            created_at: chrono::Utc::now(),
        },
        key_shares: result.shares.into_iter().map(|s| odamp_shared::KeyShare {
            share_id: s.share_id,
            encrypted_share: s.encrypted_share,
        }).collect(),
        warning: "Store these key shares securely. They will NOT be available again.",
    }))
}

pub async fn get_wallet(
    State(_state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // Query DB for wallet by ID
    // TODO: implement
    Err((axum::http::StatusCode::NOT_IMPLEMENTED, "Not yet implemented".into()))
}   