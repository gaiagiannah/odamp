use axum::extract::State;
use axum::Json;
use odamp_shared::{ScreenRequest, ScreenResponse};
use crate::state::AppState;

pub async fn screen(
    State(state): State<AppState>,
    Json(req): Json<ScreenRequest>,
) -> Result<Json<ScreenResponse>, (axum::http::StatusCode, String)> {
    let compliance = state.compliance.lock().unwrap();
    let result = compliance.screen(&req.address)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Log to compliance_logs
    // TODO: insert into DB

    Ok(Json(result))
}   