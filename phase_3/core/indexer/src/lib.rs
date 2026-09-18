//! ODAMP On-Chain Indexer
//!
//! Two genuinely real data sources, no mocking:
//!
//! 1. `EvmClient` reads balances directly from any EVM-compatible
//!    JSON-RPC endpoint you point it at (a public RPC, your own
//!    Alchemy/Infura/QuickNode endpoint, etc.) — native-token
//!    balances via `eth_getBalance`, and ERC-20 balances via a raw
//!    `eth_call` to `balanceOf(address)` (function selector
//!    `0x70a08231`), decoded from the returned hex.
//!
//! 2. `PriceClient` reads real USD prices from CoinGecko's public
//!    `/simple/price` endpoint, which does not require an API key
//!    for reasonable request volumes as of this writing (CoinGecko
//!    may rate-limit or require a key under load — see
//!    `PriceClient::new` docs).
//!
//! Neither of these was testable in the sandbox this was written in
//! (no network access there) — this is real code written against
//! real, documented APIs, but you are the first person to actually
//! run it against the live network. Expect to iterate.

use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("unexpected response shape: {0}")]
    UnexpectedResponse(String),

    #[error("hex parse error: {0}")]
    HexParse(String),

    #[error("price not found for id: {0}")]
    PriceNotFound(String),
}

pub struct EvmClient {
    rpc_url: String,
    http: reqwest::Client,
}

impl EvmClient {
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            http: reqwest::Client::new(),
        }
    }

    /// Native-token balance (ETH, MATIC, BNB, etc. depending on the
    /// chain `rpc_url` points at), returned as a decimal value
    /// already divided by 10^18 (the standard native-token decimals
    /// for EVM chains).
    pub async fn native_balance(&self, address: &str) -> Result<f64, IndexerError> {
        let result = self
            .rpc_call("eth_getBalance", json!([address, "latest"]))
            .await?;

        let hex_str = result
            .as_str()
            .ok_or_else(|| IndexerError::UnexpectedResponse(result.to_string()))?;

        wei_hex_to_decimal(hex_str, 18)
    }

    /// ERC-20 token balance for `holder_address` on `token_contract`,
    /// divided by `10^decimals` (the caller must know the token's
    /// decimals — e.g. 6 for USDC, 18 for most others; this is not
    /// fetched automatically in Phase 2, that's a reasonable Phase 3
    /// addition via the token's `decimals()` method).
    pub async fn erc20_balance(
        &self,
        token_contract: &str,
        holder_address: &str,
        decimals: u8,
    ) -> Result<f64, IndexerError> {
        // balanceOf(address) selector: 0x70a08231, followed by the
        // holder address left-padded to 32 bytes.
        let holder_no_prefix = holder_address.trim_start_matches("0x");
        let padded_holder = format!("{:0>64}", holder_no_prefix);
        let call_data = format!("0x70a08231{}", padded_holder);

        let call_params = json!([
            { "to": token_contract, "data": call_data },
            "latest"
        ]);

        let result = self.rpc_call("eth_call", call_params).await?;
        let hex_str = result
            .as_str()
            .ok_or_else(|| IndexerError::UnexpectedResponse(result.to_string()))?;

        wei_hex_to_decimal(hex_str, decimals)
    }

    async fn rpc_call(&self, method: &str, params: Value) -> Result<Value, IndexerError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let response: Value = self
            .http
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if let Some(error) = response.get("error") {
            return Err(IndexerError::Rpc(error.to_string()));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| IndexerError::UnexpectedResponse(response.to_string()))
    }
}

/// Convert a hex-encoded wei-style integer (e.g. "0x1bc16d674ec80000")
/// into a decimal f64 divided by 10^decimals.
///
/// Uses u128 for the integer parse — sufficient for any realistic
/// token balance (u128::MAX is ~3.4e38; even a supply of
/// 10^18 tokens at 18 decimals is ~10^36). Division into f64 loses
/// precision at the very low end, which is acceptable for portfolio
/// *display* purposes but NOT for anything computing exact settlement
/// amounts — those should stay in integer wei/base-unit form.
fn wei_hex_to_decimal(hex_str: &str, decimals: u8) -> Result<f64, IndexerError> {
    let cleaned = hex_str.trim_start_matches("0x");
    let cleaned = if cleaned.is_empty() { "0" } else { cleaned };

    let raw = u128::from_str_radix(cleaned, 16)
        .map_err(|e| IndexerError::HexParse(format!("{e} (input: {hex_str})")))?;

    let divisor = 10f64.powi(decimals as i32);
    Ok(raw as f64 / divisor)
}

/// Real USD price lookups via CoinGecko's public API.
pub struct PriceClient {
    http: reqwest::Client,
    base_url: String,
}

impl PriceClient {
    /// `base_url` defaults to CoinGecko's public API. If you have a
    /// CoinGecko Pro API key, point `base_url` at
    /// "https://pro-api.coingecko.com/api/v3" and add the
    /// `x-cg-pro-api-key` header yourself (not implemented here —
    /// Phase 3 concern if rate limits become a problem).
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: "https://api.coingecko.com/api/v3".to_string(),
        }
    }

    /// `coingecko_id` is CoinGecko's internal slug, not the ticker —
    /// e.g. "bitcoin", not "BTC"; "pax-gold" for PAXG. Look these up
    /// at https://api.coingecko.com/api/v3/coins/list if unsure.
    pub async fn usd_price(&self, coingecko_id: &str) -> Result<f64, IndexerError> {
        let url = format!(
            "{}/simple/price?ids={}&vs_currencies=usd",
            self.base_url, coingecko_id
        );

        let response: Value = self.http.get(&url).send().await?.json().await?;

        response
            .get(coingecko_id)
            .and_then(|v| v.get("usd"))
            .and_then(|v| v.as_f64())
            .ok_or_else(|| IndexerError::PriceNotFound(coingecko_id.to_string()))
    }
}

impl Default for PriceClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_wei_hex_to_decimal() {
        // 1 ETH = 10^18 wei = 0xde0b6b3a7640000
        let result = wei_hex_to_decimal("0xde0b6b3a7640000", 18).unwrap();
        assert!((result - 1.0).abs() < 1e-9);
    }

    #[test]
    fn decodes_zero_balance() {
        let result = wei_hex_to_decimal("0x0", 18).unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn decodes_usdc_style_6_decimals() {
        // 1,000,000 base units at 6 decimals = 1.0 USDC = 0xf4240
        let result = wei_hex_to_decimal("0xf4240", 6).unwrap();
        assert!((result - 1.0).abs() < 1e-9);
    }
}
