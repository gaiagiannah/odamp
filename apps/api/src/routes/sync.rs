use axum::extract::State;
use axum::Json;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct SyncTarget {
    pub wallet_address: String,
    pub chain: String,
    pub tokens: Vec<String>, // coingecko IDs or "NATIVE"
}

#[derive(serde::Deserialize)]
pub struct SyncRequest {
    pub wallet_id: uuid::Uuid,
    pub targets: Vec<SyncTarget>,
}

pub async fn sync(
    State(state): State<AppState>,
    Json(req): Json<SyncRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let mut results = Vec::new();

    for target in &req.targets {
        // Get RPC URL for chain
        let chain_config = odamp_shared::get_chain(&target.chain)
            .ok_or((axum::http::StatusCode::BAD_REQUEST, format!("Unknown chain: {}", target.chain)))?;

        let rpc_url = std::env::var(chain_config.rpc_env_var)
            .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("RPC URL not set for {}", chain_config.rpc_env_var)))?;

        let evm = odamp_indexer::EvmClient::new(&rpc_url);

        // Get native balance
        if target.tokens.iter().any(|t| t == "NATIVE") {
            if let Ok(native_balance) = evm.get_native_balance(&target.wallet_address).await {
                results.push(serde_json::json!({
                    "token": "NATIVE",
                    "balance": native_balance,
                    "chain": target.chain,
                }));
            }
        }

        // Get ERC-20 balances (for known tokens)
        // TODO: look up token addresses from tracked_tokens table
    }

    Ok(Json(serde_json::json!({
        "synced": results.len(),
        "results": results,
    })))
}   