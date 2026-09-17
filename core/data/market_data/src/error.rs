use thiserror::Error;

#[derive(Debug, Error)]
pub enum MarketDataError {
    #[error("Price feed unavailable for {symbol}: {reason}")]
    FeedUnavailable { symbol: String, reason: String },

    #[error("No price data found for {0}")]
    NoData(String),

    #[error("Stale data: last update was {age_secs}s ago (max: {max_secs}s)")]
    StaleData { age_secs: u64, max_secs: u64 },

    #[error("Oracle deviation exceeded threshold: {deviation_pct}% > {max_pct}%")]
    OracleDeviation { deviation_pct: f64, max_pct: f64 },

    #[error("API rate limit exceeded for {source}")]
    RateLimited { source: String },

    #[error("Commodity data unavailable: {0}")]
    CommodityUnavailable(String),

    #[error("Insufficient data points for calculation: have {have}, need {need}")]
    InsufficientData { have: usize, need: usize },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}   use thiserror::Error;

#[derive(Debug, Error)]
pub enum MarketDataError {
    #[error("Price feed unavailable for {symbol}: {reason}")]
    FeedUnavailable { symbol: String, reason: String },

    #[error("No price data found for {0}")]
    NoData(String),

    #[error("Stale data: last update was {age_secs}s ago (max: {max_secs}s)")]
    StaleData { age_secs: u64, max_secs: u64 },

    #[error("Oracle deviation exceeded threshold: {deviation_pct}% > {max_pct}%")]
    OracleDeviation { deviation_pct: f64, max_pct: f64 },

    #[error("API rate limit exceeded for {source}")]
    RateLimited { source: String },

    #[error("Commodity data unavailable: {0}")]
    CommodityUnavailable(String),

    #[error("Insufficient data points for calculation: have {have}, need {need}")]
    InsufficientData { have: usize, need: usize },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}   