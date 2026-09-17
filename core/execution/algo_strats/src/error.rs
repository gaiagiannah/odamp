use thiserror::Error;

#[derive(Debug, Error)]
pub enum StrategyError {
    #[error("Strategy {id} not found")]
    NotFound(uuid::Uuid),

    #[error("Strategy {id} is already stopped")]
    AlreadyStopped(uuid::Uuid),

    #[error("Signal would exceed position limit: {amount} > {limit}")]
    ExceedsPositionLimit { amount: f64, limit: f64 },

    #[error("Daily spend limit reached: {spent} >= {limit}")]
    DailyLimitReached { spent: f64, limit: f64 },

    #[error("Insufficient balance for strategy execution: need {needed}, have {available}")]
    InsufficientBalance { needed: f64, available: f64 },

    #[error("Strategy parameters invalid: {0}")]
    InvalidParams(String),

    #[error("Market data unavailable for strategy {id}")]
    NoMarketData(uuid::Uuid),

    #[error("Circuit breaker triggered: {0}")]
    CircuitBreaker(String),
}   