//! Coinbase Advanced Trade API implementation.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::exchange::{Exchange, ExchangeConfig, OrderSide, OrderType, Balance, Ticker, OrderBook, WsMessage};
use crate::orders::Order;
use crate::error::CexError;
use crate::auth::sign_hmac;

pub struct CoinbaseExchange {
    config: ExchangeConfig,
    client: reqwest::Client,
}

impl CoinbaseExchange {
    pub fn new(config: ExchangeConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    fn auth_headers(&self, method: &str, path: &str, body: &str) -> Vec<(String, String)> {
        let timestamp = chrono::Utc::now().timestamp_millis().to_string();
        let signature = sign_hmac(
            &self.config.api_secret,
            &format!("{}{}{}{}", timestamp, method, path, body),
        );

        vec![
            ("CB-ACCESS-KEY".into(), self.config.api_key.clone()),
            ("CB-ACCESS-SIGN".into(), signature),
            ("CB-ACCESS-TIMESTAMP".into(), timestamp),
            ("CB-ACCESS-PROJECT".into(), "odamp".into()),
            ("Content-Type".into(), "application/json".into()),
        ]
    }
}

#[async_trait]
impl Exchange for CoinbaseExchange {
    fn name(&self) -> &str {
        "coinbase"
    }

    async fn place_order(&self, order: &Order) -> Result<Order, CexError> {
        let path = "/api/v3/brokerage/orders";
        let body = serde_json::json!({
            "client_order_id": order.id,
            "product_id": order.symbol,
            "side": if order.side == OrderSide::Buy { "BUY" } else { "SELL" },
            "order_config": match &order.order_type {
                OrderType::Market => {
                    serde_json::json!({ "market_market_ioc_config": { "quote_size": order.amount.to_string() } })
                }
                OrderType::Limit { price } => {
                    serde_json::json!({ "limit_limit_gtc_config": {
                        "limit_price": price.to_string(),
                        "base_size": order.amount.to_string()
                    }})
                }
                _ => return Err(CexError::OrderRejected {
                    exchange: "coinbase".into(),
                    reason: "Unsupported order type".into(),
                }),
            },
        });

        let body_str = body.to_string();
        let headers = self.auth_headers("POST", path, &body_str);

        let response = self.client
            .post(format!("{}{}", self.config.base_url, path))
            .headers(headers.into_iter().collect())
            .json(&body)
            .send()
            .await
            .map_err(|e| CexError::Http(e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CexError::ApiError {
                exchange: "coinbase".into(),
                status: status.as_u16(),
                body,
            });
        }

        Ok(order.clone())
    }

    async fn cancel_order(&self, order_id: &str) -> Result<(), CexError> {
        let path = format!("/api/v3/brokerage/orders/{}", order_id);
        let headers = self.auth_headers("DELETE", &path, "");

        let response = self.client
            .delete(format!("{}{}", self.config.base_url, path))
            .headers(headers.into_iter().collect())
            .send()
            .await
            .map_err(|e| CexError::Http(e))?;

        if !response.status().is_success() {
            return Err(CexError::OrderNotFound(order_id.to_string()));
        }

        Ok(())
    }

    async fn get_order(&self, order_id: &str) -> Result<Order, CexError> {
        Err(CexError::OrderNotFound(order_id.to_string()))
    }

    async fn open_orders(&self) -> Result<Vec<Order>, CexError> {
        Ok(vec![])
    }

    async fn balances(&self) -> Result<Vec<Balance>, CexError> {
        let path = "/api/v3/brokerage/accounts";
        let headers = self.auth_headers("GET", path, "");

        let response = self.client
            .get(format!("{}{}", self.config.base_url, path))
            .headers(headers.into_iter().collect())
            .send()
            .await
            .map_err(|e| CexError::Http(e))?;

        Ok(vec![])
    }

    async fn ticker(&self, symbol: &str) -> Result<Ticker, CexError> {
        Ok(Ticker {
            symbol: symbol.to_string(),
            bid: 0.0,
            ask: 0.0,
            last: 0.0,
            volume_24h: 0.0,
            change_24h_pct: 0.0,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn order_book(&self, symbol: &str, depth: u32) -> Result<OrderBook, CexError> {
        Ok(OrderBook {
            symbol: symbol.to_string(),
            bids: vec![],
            asks: vec![],
            timestamp: chrono::Utc::now(),
        })
    }

    async fn subscribe(&self, channels: Vec<String>) -> Result<tokio::sync::mpsc::Receiver<WsMessage>, CexError> {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        // Production: connect WebSocket to self.config.ws_url
        // Subscribe to channels, forward messages to tx
        drop(tx);
        Ok(rx)
    }
}   