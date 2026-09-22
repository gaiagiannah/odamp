use axum::extract::State;
use axum::Json;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct AnalyzeRequest {
    pub wallet_id: uuid::Uuid,
}

pub async fn analyze(
    State(state): State<AppState>,
    Json(req): Json<AnalyzeRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let ai_url = std::env::var("AI_AGENT_URL")
        .unwrap_or_else(|_| "http://localhost:8000".into());

    // Get portfolio state from DB
    // Forward to AI agent sidecar
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/analyze", ai_url))
        .json(&serde_json::json!({ "wallet_id": req.wallet_id.to_string() }))
        .send()
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, format!("AI agent unreachable: {}", e)))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))?;

    Ok(Json(response))
}   