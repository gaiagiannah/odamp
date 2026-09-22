use axum::extract::{Path, State};
use axum::Json;
use crate::state::AppState;

pub async fn get_portfolio(
    State(state): State<AppState>,
    Path(wallet_id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // Query positions from DB
    let rows: Vec<(String, String, String, f64, f64)> = sqlx::query_as(
        "SELECT symbol, chain, balance::TEXT, price_usd, value_usd FROM positions WHERE wallet_id = $1 ORDER BY value_usd DESC"
    )
    .bind(wallet_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total: f64 = rows.iter().map(|r| r.4).sum();

    Ok(Json(serde_json::json!({
        "wallet_id": wallet_id.to_string(),
        "total_value_usd": total,
        "positions": rows.iter().map(|r| serde_json::json!({
            "symbol": r.0,
            "chain": r.1,
            "balance": r.2,
            "price_usd": r.3,
            "value_usd": r.4,
        })).collect::<Vec<_>>(),
    })))
}   