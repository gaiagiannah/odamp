use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataConfig {
    pub max_staleness_secs: u64,
    pub max_oracle_deviation_pct: f64,
    pub price_sources: Vec<String>,
    pub commodity_sources: Vec<String>,
    pub polling_interval_ms: u64,
    pub volatility_history_hours: u32,
}

impl MarketDataConfig {
    pub fn default_dev() -> Self {
        Self {
            max_staleness_secs: 30,
            max_oracle_deviation_pct: 2.0,
            price_sources: vec![
                "chainlink".into(),
                "coinbase".into(),
                "binance".into(),
                "kraken".into(),
            ],
            commodity_sources: vec![
                "chainlink".into(),
                "lme".into(),
                "comex".into(),
            ],
            polling_interval_ms: 5_000,
            volatility_history_hours: 720, // 30 days
        }
    }
}

impl Default for MarketDataConfig {
    fn default() -> Self {
        Self::default_dev()
    }
}   