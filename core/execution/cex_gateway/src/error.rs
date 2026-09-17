use thiserror::Error;

#[derive(Debug, Error)]
pub enum CexError {
    #[error("Authentication failed for {exchange}: {reason}")]
    AuthFailed { exchange: String, reason: String },

    #[error("Order rejected by {exchange}: {reason}")]
    OrderRejected { exchange: String, reason: String },

    #[error("Insufficient balance on {exchange}: need {needed}, have {available}")]
    InsufficientBalance { exchange: String, needed: f64, available: f64 },

    #[error("Rate limit exceeded on {exchange}. Retry in {retry_ms}ms")]
    RateLimited { exchange: String, retry_ms: u64 },

    #[error("API error from {exchange}: HTTP {status} - {body}")]
    ApiError { exchange: String, status: u16, body: String },

    #[error("WebSocket disconnected from {exchange}: {reason}")]
    WebSocketDisconnected { exchange: String, reason: String },

    #[error("Order not found: {0}")]
    OrderNotFound(String),

    #[error("Exchange {exchange} is down")]
    ExchangeDown(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}   