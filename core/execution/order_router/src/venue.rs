//! Venue abstraction: unified interface for CEXs, DEXs, OTC desks.

use serde::{Deserialize, Serialize};

/// Type of trading venue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VenueType {
    Cex,
    Dex,
    Otc,
    CrossChain,
}

/// A trading venue (exchange, DEX protocol, OTC desk).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Venue {
    pub id: String,
    pub name: String,
    pub venue_type: VenueType,
    pub chain: Option<String>,      // for DEXs
    pub base_fee_bps: f64,          // base trading fee in basis points
    pub max_order_usd: f64,         // max single order size
    pub min_order_usd: f64,         // min single order size
    pub is_available: bool,
    pub latency_ms: u64,            // typical API latency
    pub reliability_score: f64,     // 0-100, uptime history
}

impl Venue {
    pub fn coinbase() -> Self {
        Self {
            id: "coinbase".into(),
            name: "Coinbase".into(),
            venue_type: VenueType::Cex,
            chain: None,
            base_fee_bps: 120.0,
            max_order_usd: 1_000_000.0,
            min_order_usd: 1.0,
            is_available: true,
            latency_ms: 50,
            reliability_score: 99.9,
        }
    }

    pub fn kraken() -> Self {
        Self {
            id: "kraken".into(),
            name: "Kraken".into(),
            venue_type: VenueType::Cex,
            chain: None,
            base_fee_bps: 26.0,
            max_order_usd: 500_000.0,
            min_order_usd: 1.0,
            is_available: true,
            latency_ms: 45,
            reliability_score: 99.8,
        }
    }

    pub fn binance() -> Self {
        Self {
            id: "binance".into(),
            name: "Binance".into(),
            venue_type: VenueType::Cex,
            chain: None,
            base_fee_bps: 10.0,
            max_order_usd: 5_000_000.0,
            min_order_usd: 5.0,
            is_available: true,
            latency_ms: 30,
            reliability_score: 99.95,
        }
    }

    pub fn okx() -> Self {
        Self {
            id: "okx".into(),
            name: "OKX".into(),
            venue_type: VenueType::Cex,
            chain: None,
            base_fee_bps: 10.0,
            max_order_usd: 2_000_000.0,
            min_order_usd: 1.0,
            is_available: true,
            latency_ms: 40,
            reliability_score: 99.9,
        }
    }

    pub fn uniswap_v3() -> Self {
        Self {
            id: "uniswap-v3".into(),
            name: "Uniswap V3".into(),
            venue_type: VenueType::Dex,
            chain: Some("ethereum".into()),
            base_fee_bps: 30.0,
            max_order_usd: 10_000_000.0,
            min_order_usd: 1.0,
            is_available: true,
            latency_ms: 2000, // block time
            reliability_score: 99.5,
        }
    }

    pub fn curve() -> Self {
        Self {
            id: "curve".into(),
            name: "Curve".into(),
            venue_type: VenueType::Dex,
            chain: Some("ethereum".into()),
            base_fee_bps: 4.0,
            max_order_usd: 5_000_000.0,
            min_order_usd: 1.0,
            is_available: true,
            latency_ms: 2000,
            reliability_score: 99.3,
        }
    }

    pub fn jupiter() -> Self {
        Self {
            id: "jupiter".into(),
            name: "Jupiter".into(),
            venue_type: VenueType::Dex,
            chain: Some("solana".into()),
            base_fee_bps: 25.0,
            max_order_usd: 5_000_000.0,
            min_order_usd: 1.0,
            is_available: true,
            latency_ms: 400,
            reliability_score: 99.7,
        }
    }

    pub fn oneinch() -> Self {
        Self {
            id: "1inch".into(),
            name: "1inch".into(),
            venue_type: VenueType::Dex,
            chain: Some("multi".into()),
            base_fee_bps: 15.0,
            max_order_usd: 10_000_000.0,
            min_order_usd: 1.0,
            is_available: true,
            latency_ms: 1500,
            reliability_score: 99.6,
        }
    }
}

/// A quote from a venue for a specific order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueQuote {
    pub venue_id: String,
    pub symbol: String,
    pub side: Side,
    pub amount: f64,
    pub price: f64,
    pub fee_bps: f64,
    pub estimated_slippage_bps: f64,
    pub liquidity_available: f64,
    pub execution_time_ms: u64,
    pub is_guaranteed: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl VenueQuote {
    pub fn total_cost(&self) -> f64 {
        let base = self.amount * self.price;
        let fee = base * (self.fee_bps / 10_000.0);
        let slippage = base * (self.estimated_slippage_bps / 10_000.0);
        match self.side {
            Side::Buy => base + fee + slippage,
            Side::Sell => base - fee - slippage,
        }
    }

    pub fn effective_price(&self) -> f64 {
        self.total_cost() / self.amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_venue_defaults() {
        let cb = Venue::coinbase();
        assert_eq!(cb.venue_type, VenueType::Cex);
        assert!(cb.is_available);
        assert!(cb.base_fee_bps > 0.0);
    }

    #[test]
    fn test_quote_cost() {
        let quote = VenueQuote {
            venue_id: "coinbase".into(),
            symbol: "BTC".into(),
            side: Side::Buy,
            amount: 1.0,
            price: 100_000.0,
            fee_bps: 120.0,
            estimated_slippage_bps: 5.0,
            liquidity_available: 10.0,
            execution_time_ms: 50,
            is_guaranteed: true,
            timestamp: chrono::Utc::now(),
        };

        // Cost = 100000 + 100000*0.012 + 100000*0.0005 = 100000 + 1200 + 50 = 101250
        assert!((quote.total_cost() - 101_250.0).abs() < 0.01);
    }
}   