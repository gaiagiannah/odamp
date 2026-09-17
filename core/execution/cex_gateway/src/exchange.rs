//! Exchange abstraction layer.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::orders::{Order, OrderStatus, Fill};
use crate::error::CexError;

/// Order side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit { price: f64 },
    StopLoss { trigger_price: f64 },
    StopLimit { trigger_price: f64, limit_price: f64 },
    TrailingStop { callback_pct: f64 },
}

/// Exchange configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub name: String,
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: Option<String>, // OKX, Bybit
    pub base_url: String,
    pub ws_url: String,
    pub testnet: bool,
}

/// Trait that all exchange implementations must satisfy.
#[async_trait]
pub trait Exchange: Send + Sync {
    fn name(&self) -> &str;

    /// Places a new order.
    async fn place_order(&self, order: &Order) -> Result<Order, CexError>;

    /// Cancels an existing order.
    async fn cancel_order(&self, order_id: &str) -> Result<(), CexError>;

    /// Gets current order status.
    async fn get_order(&self, order_id: &str) -> Result<Order, CexError>;

    /// Gets all open orders.
    async fn open_orders(&self) -> Result<Vec<Order>, CexError>;

    /// Gets account balances.
    async fn balances(&self) -> Result<Vec<Balance>, CexError>;

    /// Gets current ticker/price.
    async fn ticker(&self, symbol: &str) -> Result<Ticker, CexError>;

    /// Gets order book depth.
    async fn order_book(&self, symbol: &str, depth: u32) -> Result<OrderBook, CexError>;

    /// Subscribes to real-time updates via WebSocket.
    async fn subscribe(&self, channels: Vec<String>) -> Result<tokio::sync::mpsc::Receiver<WsMessage>, CexError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub currency: String,
    pub available: f64,
    pub pending: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume_24h: f64,
    pub change_24h_pct: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: Vec<(f64, f64)>, // (price, amount)
    pub asks: Vec<(f64, f64)>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessage {
    Ticker(Ticker),
    OrderUpdate(Order),
    Fill(Fill),
    BalanceUpdate(Balance),
    Ping,
    Pong,
}   