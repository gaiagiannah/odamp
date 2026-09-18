//! ODAMP API — on-chain sync endpoint (Phase 2)
//!
//! Replaces manually POSTing positions with real reads: for each
//! `SyncTarget` in the request, this fetches a real balance from an
//! EVM JSON-RPC endpoint and a real USD price from CoinGecko, then
//! upserts the resulting `Position` into Postgres.
//!
//! This does NOT auto-discover a wallet's holdings (that requires
//! either an indexing service like Alchemy's token-balances API or
//! walking every known token contract, both reasonable Phase 3
//! additions) — the caller must specify which assets to check. That
//! is a real limitation, not a simplification for its own sake: full
//! auto-discovery either costs money (hosted indexer APIs) or is slow
//! (querying token contracts one at a time), and Phase 2's job is to
//! prove the "real balance in, real position out" path works at all.

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use odamp_indexer::{EvmClient, PriceClient};
use odamp_portfolio::Position;

use crate::{db, internal_error, AppState};

#[derive(Deserialize)]
pub struct SyncTarget {
    /// JSON-RPC endpoint for the chain this asset lives on (e.g. a
    /// public Ethereum RPC, or your own Alchemy/Infura URL).
    pub rpc_url: String,
    /// Free-text label stored on the resulting position, e.g. "ethereum", "polygon".
    pub chain: String,
    /// The wallet address to check the balance of.
    pub holder_address: String,
    /// "native" for ETH/MATIC/etc., "erc20" for a token contract.
    pub kind: AssetKind,
    /// Required if kind == "erc20".
    pub token_contract: Option<String>,
    /// Required if kind == "erc20". Ignored for native assets (fixed at 18).
    pub decimals: Option<u8>,
    /// Display symbol, e.g. "ETH", "USDC", "PAXG".
    pub symbol: String,
    /// e.g. "crypto", "stablecoin", "tokenized_commodity".
    pub asset_type: String,
    /// CoinGecko's internal id for this asset (NOT the ticker) —
    /// e.g. "ethereum", "usd-coin", "pax-gold".
    pub coingecko_id: String,
    /// Average cost basis in USD per unit. Phase 2 does not compute
    /// this from transaction history yet (that's the tax-engine
    /// phase) — the caller supplies it.
    pub avg_cost_usd: f64,
}

#[derive(Deserialize)]
pub enum AssetKind {
    #[serde(rename = "native")]
    Native,
    #[serde(rename = "erc20")]
    Erc20,
}

#[derive(Serialize)]
pub struct SyncResult {
    pub symbol: String,
    pub amount: f64,
    pub current_price_usd: f64,
    pub value_usd: f64,
}

#[derive(Serialize)]
pub struct SyncResponse {
    pub synced: Vec<SyncResult>,
    pub errors: Vec<String>,
}

pub async fn sync_positions(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(targets): Json<Vec<SyncTarget>>,
) -> Result<Json<SyncResponse>, (StatusCode, String)> {
    if !db::user_exists(&state.pool, user_id)
        .await
        .map_err(internal_error)?
    {
        return Err((StatusCode::NOT_FOUND, "user not found".into()));
    }

    let price_client = PriceClient::new();
    let mut synced = Vec::new();
    let mut errors = Vec::new();

    for target in targets {
        match sync_one(&state, user_id, &price_client, &target).await {
            Ok(result) => synced.push(result),
            Err(e) => errors.push(format!("{}: {}", target.symbol, e)),
        }
    }

    Ok(Json(SyncResponse { synced, errors }))
}

async fn sync_one(
    state: &AppState,
    user_id: Uuid,
    price_client: &PriceClient,
    target: &SyncTarget,
) -> anyhow::Result<SyncResult> {
    let evm = EvmClient::new(&target.rpc_url);

    let amount = match target.kind {
        AssetKind::Native => evm.native_balance(&target.holder_address).await?,
        AssetKind::Erc20 => {
            let contract = target
                .token_contract
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("token_contract required for erc20 assets"))?;
            let decimals = target
                .decimals
                .ok_or_else(|| anyhow::anyhow!("decimals required for erc20 assets"))?;
            evm.erc20_balance(contract, &target.holder_address, decimals)
                .await?
        }
    };

    let current_price_usd = price_client.usd_price(&target.coingecko_id).await?;

    // Deterministic position id: same (user, chain, symbol) always
    // maps to the same UUID, so repeated syncs update the existing
    // row via upsert_position's ON CONFLICT instead of creating
    // duplicate rows for the same asset every time you sync.
    let namespace = Uuid::NAMESPACE_OID;
    let id_input = format!("{}:{}:{}", user_id, target.chain, target.symbol);
    let position_id = Uuid::new_v5(&namespace, id_input.as_bytes());

    let position = Position {
        id: position_id,
        asset_symbol: target.symbol.clone(),
        asset_type: target.asset_type.clone(),
        amount,
        avg_cost_usd: target.avg_cost_usd,
        current_price_usd,
    };

    db::upsert_position(&state.pool, user_id, &position).await?;

    Ok(SyncResult {
        symbol: target.symbol.clone(),
        amount,
        current_price_usd,
        value_usd: amount * current_price_usd,
    })
}
