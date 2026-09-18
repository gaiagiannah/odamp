//! ODAMP API — Database layer (Phase 1)
//!
//! Uses `sqlx::query` (runtime-checked) rather than the `sqlx::query!`
//! macro (compile-time-checked) deliberately: the macro needs a live
//! database connection or a `.sqlx/` prepared-query cache available
//! at *compile* time, which isn't available in every dev environment
//! (including the sandbox this was originally written in). Runtime
//! queries are slightly less safe (a typo in a column name fails at
//! first call, not at `cargo build`) but have no such requirement.
//! If you want compile-time checking once you have a DB running
//! locally, migrating these to `query!` is a reasonable Phase 2
//! cleanup — not required for correctness.

use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use uuid::Uuid;

use odamp_portfolio::Position;

pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("../migrations").run(pool).await?;
    Ok(())
}

// ---------- Users ----------

pub async fn create_user(pool: &PgPool, jurisdiction: &str) -> anyhow::Result<Uuid> {
    let row = sqlx::query("INSERT INTO users (jurisdiction) VALUES ($1) RETURNING id")
        .bind(jurisdiction)
        .fetch_one(pool)
        .await?;
    Ok(row.try_get("id")?)
}

pub async fn user_exists(pool: &PgPool, user_id: Uuid) -> anyhow::Result<bool> {
    let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) AS exists")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(row.try_get("exists")?)
}

// ---------- Wallets ----------
//
// Only public wallet info is ever inserted here — see the security
// crate's module doc for why. No key-share data touches this table.

pub struct WalletRecord {
    pub wallet_id: Uuid,
    pub threshold: i16,
    pub total_shares: i16,
    pub group_public_key_hex: String,
}

pub async fn insert_wallet(
    pool: &PgPool,
    user_id: Uuid,
    threshold: u16,
    total_shares: u16,
    group_public_key_hex: &str,
) -> anyhow::Result<Uuid> {
    let row = sqlx::query(
        r#"
        INSERT INTO wallets (user_id, threshold, total_shares, group_public_key_hex)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(threshold as i16)
    .bind(total_shares as i16)
    .bind(group_public_key_hex)
    .fetch_one(pool)
    .await?;
    Ok(row.try_get("id")?)
}

pub async fn get_wallet(pool: &PgPool, wallet_id: Uuid) -> anyhow::Result<Option<WalletRecord>> {
    let row = sqlx::query(
        "SELECT id, threshold, total_shares, group_public_key_hex FROM wallets WHERE id = $1",
    )
    .bind(wallet_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| WalletRecord {
        wallet_id: r.get("id"),
        threshold: r.get("threshold"),
        total_shares: r.get("total_shares"),
        group_public_key_hex: r.get("group_public_key_hex"),
    }))
}

// ---------- Positions ----------

pub async fn upsert_position(pool: &PgPool, user_id: Uuid, position: &Position) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO positions (id, user_id, asset_symbol, asset_type, amount, avg_cost_usd, current_price_usd, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
        ON CONFLICT (id) DO UPDATE SET
            amount = EXCLUDED.amount,
            avg_cost_usd = EXCLUDED.avg_cost_usd,
            current_price_usd = EXCLUDED.current_price_usd,
            updated_at = NOW()
        "#,
    )
    .bind(position.id)
    .bind(user_id)
    .bind(&position.asset_symbol)
    .bind(&position.asset_type)
    .bind(position.amount)
    .bind(position.avg_cost_usd)
    .bind(position.current_price_usd)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_positions_for_user(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Position>> {
    let rows = sqlx::query(
        "SELECT id, asset_symbol, asset_type, amount, avg_cost_usd, current_price_usd
         FROM positions WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let positions = rows
        .into_iter()
        .map(|r| Position {
            id: r.get("id"),
            asset_symbol: r.get("asset_symbol"),
            asset_type: r.get("asset_type"),
            amount: r.get("amount"),
            avg_cost_usd: r.get("avg_cost_usd"),
            current_price_usd: r.get("current_price_usd"),
        })
        .collect();

    Ok(positions)
}

// ---------- Transactions (sanctions-screening audit trail) ----------

pub async fn record_screened_transaction(
    pool: &PgPool,
    user_id: Uuid,
    wallet_id: Option<Uuid>,
    chain: &str,
    from_address: &str,
    to_address: &str,
    blocked: bool,
) -> anyhow::Result<Uuid> {
    let row = sqlx::query(
        r#"
        INSERT INTO transactions
            (user_id, wallet_id, chain, from_address, to_address, sanctions_screened, sanctions_blocked)
        VALUES ($1, $2, $3, $4, $5, TRUE, $6)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(wallet_id)
    .bind(chain)
    .bind(from_address)
    .bind(to_address)
    .bind(blocked)
    .fetch_one(pool)
    .await?;
    Ok(row.try_get("id")?)
}
