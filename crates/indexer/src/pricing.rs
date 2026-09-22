//! Price client — CoinGecko public API.
//!
//! Uses /simple/price endpoint. No API key required for
//! reasonable request volumes. Rate-limited under load.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriceError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Token not found: {0}")]
    TokenNotFound(String),
}

pub struct PriceClient {
    base_url: String,
    api_key: Option<String>,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct PriceResponse {
    // Keys are coingecko IDs, values are objects with "usd" field
    #[serde(flatten)]
    prices: HashMap<String, PriceEntry>,
}

#[derive(Debug, Deserialize)]
struct PriceEntry {
    usd: f64,
}

impl PriceClient {
    pub fn new(base_url: &str, api_key: Option<String>) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            http: reqwest::Client::new(),
        }
    }

    /// Get USD prices for a list of CoinGecko token IDs.
    ///
    /// # Example
    /// `get_prices(&["ethereum", "usd-coin", "tether"])`
    /// → `{ "ethereum": 3500.50, "usd-coin": 1.0, "tether": 0.999 }`
    pub async fn get_prices(&self, coingecko_ids: &[&str]) -> Result<HashMap<String, f64>, PriceError> {
        let ids = coingecko_ids.join(",");
        let url = format!(
            "{}/simple/price?ids={}&vs_currencies=usd",
            self.base_url, ids
        );

        let mut req = self.http.get(&url);
        if let Some(key) = &self.api_key {
            req = req.header("x-cg-demo-api-key", key);
        }

        let resp: PriceResponse = req
            .send()
            .await
            .map_err(|e| PriceError::Api(e.to_string()))?
            .json()
            .await
            .map_err(|e| PriceError::Api(e.to_string()))?;

        let mut result = HashMap::new();
        for (id, entry) in resp.prices {
            result.insert(id, entry.usd);
        }
        Ok(result)
    }

    /// Get a single token's USD price.
    pub async fn get_price(&self, coingecko_id: &str) -> Result<f64, PriceError> {
        let prices = self.get_prices(&[coingecko_id]).await?;
        prices
            .get(coingecko_id)
            .copied()
            .ok_or_else(|| PriceError::TokenNotFound(coingecko_id.to_string()))
    }
}   