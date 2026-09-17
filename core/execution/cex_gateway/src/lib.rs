//! ODAMP CEX Gateway
//!
//! Unified interface for Centralized Exchange APIs.
//! Handles authentication, order placement, position tracking,
//! and WebSocket subscriptions for real-time data.
//!
//! Supported exchanges (v1):
//! - Coinbase Advanced
//! - Kraken
//! - Binance
//! - OKX
//! - Bybit

pub mod exchange;
pub mod coinbase;
pub mod kraken;
pub mod binance;
pub mod okx;
pub mod auth;
pub mod orders;
pub mod websocket;
pub mod error;

pub use exchange::{Exchange, ExchangeConfig, OrderSide, OrderType};
pub use orders::{Order, OrderStatus, Fill};
pub use error::CexError;   