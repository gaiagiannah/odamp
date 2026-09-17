//! OKX API implementation.

use async_trait::async_trait;
use crate::exchange::{Exchange, ExchangeConfig, OrderSide, OrderType, Balance, Ticker, OrderBook, WsMessage};
use crate::orders::Order;
use crate::error::CexError;

pub struct OkxExchange {
    config: ExchangeConfig,
    client: reqwest::Client,
}

impl OkxExchange {
    pub fn new(config: ExchangeConfig) -> Self {
        Self { config, client: reqwest::Client::new() }
    }
}

#[async_trait]
impl Exchange for OkxExchange {
    fn name(&self) -> &str { "okx" }

    async fn place_order(&self, order: &Order) -> Result<Order, CexError> {
        // Production: POST /api/v5/trade/order with OKX-specific signing
        Ok(order.clone())
    }

    async fn cancel_order(&self, order_id: &str) -> Result<(), CexError> { Ok(()) }
    async fn get_order(&self, order_id: &str) -> Result<Order, CexError> {
        Err(CexError::OrderNotFound(order_id.to_string()))
    }
    async fn open_orders(&self) -> Result<Vec<Order>, CexError> { Ok(vec![]) }
    async fn balances(&self) -> Result<Vec<Balance>, CexError> { Ok(vec![]) }

    async fn ticker(&self, symbol: &str) -> Result<Ticker, CexError> {
        Ok(Ticker {
            symbol: symbol.to_string(), bid: 0.0, ask: 0.0, last: 0.0,
            volume_24h: 0.0, change_24h_pct: 0.0, timestamp: chrono::Utc::now(),
        })
    }

    async fn order_book(&self, symbol: &str, depth: u32) -> Result<OrderBook, CexError> {
        Ok(OrderBook {
            symbol: symbol.to_string(), bids: vec![], asks: vec![], timestamp: chrono::Utc::now(),
        })
    }

    async fn subscribe(&self, _channels: Vec<String>) -> Result<tokio::sync::mpsc::Receiver<WsMessage>, CexError> {
        let (_tx, rx) = tokio::sync::mpsc::channel(1024);
        Ok(rx)
    }
}   