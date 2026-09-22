//! Position data model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub chain: String,
    pub token_address: Option<String>, // None for native tokens
    pub symbol: String,
    pub balance: String,               // human-readable decimal string
    pub price_usd: f64,
    pub value_usd: f64,
    pub updated_at: DateTime<Utc>,
}

impl Position {
    pub fn new(
        wallet_id: Uuid,
        chain: &str,
        token_address: Option<String>,
        symbol: &str,
        balance: &str,
        price_usd: f64,
    ) -> Self {
        let balance_f64: f64 = balance.parse().unwrap_or(0.0);
        Self {
            id: Uuid::new_v4(),
            wallet_id,
            chain: chain.to_string(),
            token_address,
            symbol: symbol.to_string(),
            balance: balance.to_string(),
            price_usd,
            value_usd: balance_f64 * price_usd,
            updated_at: Utc::now(),
        }
    }
}   