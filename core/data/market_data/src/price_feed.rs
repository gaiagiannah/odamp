//! Real-time price feed aggregation from multiple sources.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use crate::error::MarketDataError;

/// Source of a price quote.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PriceSource {
    ChainlinkOracle,
    Exchange { name: String },
    DexPool { protocol: String, pool: String },
    CommodityExchange { exchange: String },
    Aggregated,
}

/// A single price quote from a source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceQuote {
    pub symbol: String,
    pub price_usd: f64,
    pub source: PriceSource,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub confidence: f64, // 0.0 - 1.0
}

/// Aggregated price with confidence interval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedPrice {
    pub symbol: String,
    pub price_usd: f64,
    pub low: f64,
    pub high: f64,
    pub sources: Vec<PriceQuote>,
    pub deviation_pct: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Manages price feeds for all tracked symbols.
pub struct PriceFeed {
    prices: RwLock<HashMap<String, AggregatedPrice>>,
    max_staleness: Duration,
    max_deviation_pct: f64,
}

impl PriceFeed {
    pub fn new(max_staleness_secs: u64, max_deviation_pct: f64) -> Self {
        Self {
            prices: RwLock::new(HashMap::new()),
            max_staleness: Duration::from_secs(max_staleness_secs),
            max_deviation_pct,
        }
    }

    /// Updates the price for a symbol from a new quote.
    pub fn update(&self, quote: PriceQuote) {
        let mut prices = self.prices.write();
        let entry = prices
            .entry(quote.symbol.clone())
            .or_insert_with(|| AggregatedPrice {
                symbol: quote.symbol.clone(),
                price_usd: quote.price_usd,
                low: quote.price_usd,
                high: quote.price_usd,
                sources: vec![],
                deviation_pct: 0.0,
                last_updated: quote.timestamp,
            });

        entry.sources.push(quote.clone());
        // Keep only last 10 quotes per symbol
        if entry.sources.len() > 10 {
            entry.sources.remove(0);
        }

        // Recalculate aggregate (weighted by confidence)
        let total_weight: f64 = entry.sources.iter().map(|s| s.confidence).sum();
        if total_weight > 0.0 {
            entry.price_usd = entry
                .sources
                .iter()
                .map(|s| s.price_usd * s.confidence)
                .sum::<f64>()
                / total_weight;
        }

        entry.low = entry.sources.iter().map(|s| s.price_usd).fold(f64::MAX, f64::min);
        entry.high = entry.sources.iter().map(|s| s.price_usd).fold(f64::MIN, f64::max);

        // Calculate deviation
        if entry.low > 0.0 {
            entry.deviation_pct = ((entry.high - entry.low) / entry.low) * 100.0;
        }

        entry.last_updated = chrono::Utc::now();
    }

    /// Gets the current aggregated price for a symbol.
    pub fn get(&self, symbol: &str) -> Result<AggregatedPrice, MarketDataError> {
        let prices = self.prices.read();
        let price = prices
            .get(symbol)
            .ok_or_else(|| MarketDataError::NoData(symbol.to_string()))?;

        // Check staleness
        let age = chrono::Utc::now().signed_duration_since(price.last_updated).num_seconds() as u64;
        if age > self.max_staleness.as_secs() {
            return Err(MarketDataError::StaleData {
                age_secs: age,
                max_secs: self.max_staleness.as_secs(),
            });
        }

        // Check deviation
        if price.deviation_pct > self.max_deviation_pct {
            return Err(MarketDataError::OracleDeviation {
                deviation_pct: price.deviation_pct,
                max_pct: self.max_deviation_pct,
            });
        }

        Ok(price.clone())
    }

    /// Gets prices for multiple symbols.
    pub fn get_many(&self, symbols: &[&str]) -> Result<Vec<AggregatedPrice>, MarketDataError> {
        symbols.iter().map(|s| self.get(s)).collect()
    }

    /// Returns all currently tracked symbols.
    pub fn symbols(&self) -> Vec<String> {
        self.prices.read().keys().cloned().collect()
    }
}

impl Default for PriceFeed {
    fn default() -> Self {
        Self::new(30, 2.0) // 30s max staleness, 2% max deviation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_aggregation() {
        let feed = PriceFeed::new(30, 5.0);

        feed.update(PriceQuote {
            symbol: "BTC".into(),
            price_usd: 100_000.0,
            source: PriceSource::Exchange { name: "Coinbase".into() },
            timestamp: chrono::Utc::now(),
            confidence: 0.9,
        });

        feed.update(PriceQuote {
            symbol: "BTC".into(),
            price_usd: 100_100.0,
            source: PriceSource::ChainlinkOracle,
            timestamp: chrono::Utc::now(),
            confidence: 1.0,
        });

        let price = feed.get("BTC").unwrap();
        assert!(price.price_usd > 100_000.0);
        assert!(price.price_usd < 100_100.0);
        assert_eq!(price.sources.len(), 2);
        assert!(price.deviation_pct < 0.2);
    }

    #[test]
    fn test_unknown_symbol() {
        let feed = PriceFeed::default();
        assert!(feed.get("UNKNOWN").is_err());
    }
}   