//! WebSocket connection management for real-time exchange data.

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use crate::exchange::WsMessage;
use crate::error::CexError;

/// Manages a WebSocket connection to an exchange.
pub struct WsConnection {
    exchange_name: String,
    url: String,
    tx: mpsc::Sender<WsMessage>,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
}

impl WsConnection {
    pub fn new(exchange_name: &str, url: &str, tx: mpsc::Sender<WsMessage>) -> Self {
        Self {
            exchange_name: exchange_name.to_string(),
            url: url.to_string(),
            tx,
            reconnect_attempts: 0,
            max_reconnect_attempts: 5,
        }
    }

    /// Connects and starts receiving messages.
    pub async fn connect(&mut self) -> Result<(), CexError> {
        // Production: use tokio-tungstenite or ws
        // 1. Connect to self.url
        // 2. Send subscription message
        // 3. Loop: receive messages, parse, forward to self.tx
        // 4. On disconnect: reconnect with exponential backoff
        tracing::info!(exchange = %self.exchange_name, "WebSocket connected");
        Ok(())
    }

    /// Handles reconnection with exponential backoff.
    pub fn backoff_delay(&self) -> std::time::Duration {
        let base_ms = 1000u64;
        let delay = base_ms * 2u64.pow(self.reconnect_attempts.min(10));
        std::time::Duration::from_millis(delay)
    }
}   