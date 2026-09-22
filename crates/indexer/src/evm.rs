//! EVM chain client.
//!
//! Reads native token balances via eth_getBalance and
//! ERC-20 balances via eth_call to balanceOf(address)
//! (function selector 0x70a08231).

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EvmError {
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Decoding error: {0}")]
    Decode(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
}

/// A client for reading balances from an EVM-compatible chain.
pub struct EvmClient {
    rpc_url: String,
    http: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: T,
    #[serde(default)]
    error: Option<serde_json::Value>,
}

impl EvmClient {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Get native token balance (e.g., ETH, MATIC).
    /// Returns the balance in the smallest unit (wei).
    pub async fn get_native_balance(&self, address: &str) -> Result<String, EvmError> {
        let req = RpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "eth_getBalance".into(),
            params: vec![
                serde_json::json!(address),
                serde_json::json!("latest"),
            ],
        };

        let resp: RpcResponse<String> = self
            .http
            .post(&self.rpc_url)
            .json(&req)
            .send()
            .await
            .map_err(|e| EvmError::Rpc(e.to_string()))?
            .json()
            .await
            .map_err(|e| EvmError::Rpc(e.to_string()))?;

        if let Some(err) = resp.error {
            return Err(EvmError::Rpc(err.to_string()));
        }

        // Convert hex to decimal string
        let hex_str = resp.result.trim_start_matches("0x");
        let wei = u128::from_str_radix(hex_str, 16)
            .map_err(|e| EvmError::Decode(e.to_string()))?;
        Ok(wei.to_string())
    }

    /// Get ERC-20 token balance.
    /// Calls balanceOf(address) — selector 0x70a08231.
    pub async fn get_erc20_balance(&self, token_address: &str, wallet_address: &str) -> Result<String, EvmError> {
        // Encode: 0x70a08231 + padded address
        let padded_address = format!("{:064}", hex::encode(wallet_address.trim_start_matches("0x").as_bytes()));
        // Actually, we need to pad the hex address to 32 bytes
        let addr_hex = wallet_address.trim_start_matches("0x");
        let padded = format!("{:064}", addr_hex);
        let data = format!("0x70a08231{}", padded);

        let req = RpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "eth_call".into(),
            params: vec![
                serde_json::json!({
                    "to": token_address,
                    "data": data,
                }),
                serde_json::json!("latest"),
            ],
        };

        let resp: RpcResponse<String> = self
            .http
            .post(&self.rpc_url)
            .json(&req)
            .send()
            .await
            .map_err(|e| EvmError::Rpc(e.to_string()))?
            .json()
            .await
            .map_err(|e| EvmError::Rpc(e.to_string()))?;

        if let Some(err) = resp.error {
            return Err(EvmError::Rpc(err.to_string()));
        }

        let hex_str = resp.result.trim_start_matches("0x");
        if hex_str.is_empty() || hex_str == "0" {
            return Ok("0".to_string());
        }
        let value = u128::from_str_radix(hex_str, 16)
            .map_err(|e| EvmError::Decode(e.to_string()))?;
        Ok(value.to_string())
    }
}   