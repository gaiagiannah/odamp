use axum::extract::{Path, State};
use axum::Json;
use odamp_shared::{SignRequest, SignResponse};
use crate::state::AppState;

pub async fn sign(
    State(_state): State<AppState>,
    Path(_wallet_id): Path<uuid::Uuid>,
    Json(req): Json<SignRequest>,
) -> Result<Json<SignResponse>, (axum::http::StatusCode, String)> {
    // 1. OFAC screen (if message contains an address)
    // 2. FROST sign with provided shares
    // 3. Log to signing_logs
    // TODO: implement full flow
    Err((axum::http::StatusCode::NOT_IMPLEMENTED, "FROST signing: wire up security-core".into()))
}   