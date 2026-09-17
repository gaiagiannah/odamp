//! Valuation engine: converts on-chain positions to USD values
//! using real-time market data.

use serde::{Deserialize, Serialize};
use odamp_indexer::positions::{Position, AssetType};
use odamp_indexer::chain::ChainId;
use odamp_market_data::price_feed::{PriceFeed, AggregatedPrice};
use crate::error::PortfolioError;

/// Valuation for a single position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionValuation {
    pub position: Position,
    pub price_usd: f64,
    pub value_usd: f64,
    pub cost_basis_usd: Option<f64>,
    pub unrealized_gain_usd: Option<f64>,
    pub unrealized_gain_pct: Option<f64>,
    pub source: String,
}

/// Calculates USD valuations for all positions.
pub struct ValuationEngine {
    price_feed: std::sync::Arc<PriceFeed>,
}

impl ValuationEngine {
    pub fn new(price_feed: std::sync::Arc<PriceFeed>) -> Self {
        Self { price_feed }
    }

    /// Values a single position.
    pub fn value_position(&self, position: &Position) -> Result<PositionValuation, PortfolioError> {
        let price = self.price_feed.get(&position.asset_symbol)
            .map(|p: AggregatedPrice| p.price_usd)
            .unwrap_or(0.0);

        if price == 0.0 {
            return Err(PortfolioError::PriceUnavailable(position.asset_symbol.clone()));
        }

        let value_usd = position.amount * price;

        Ok(PositionValuation {
            position: position.clone(),
            price_usd: price,
            value_usd,
            cost_basis_usd: None, // filled by tax engine
            unrealized_gain_usd: None,
            unrealized_gain_pct: None,
            source: "aggregated".into(),
        })
    }

    /// Values all positions for a portfolio.
    pub fn value_portfolio(&self, positions: &[Position]) -> Result<Vec<PositionValuation>, PortfolioError> {
        positions.iter().map(|p| self.value_position(p)).collect()
    }

    /// Calculates total portfolio value.
    pub fn total_value(&self, positions: &[Position]) -> Result<f64, PortfolioError> {
        let valuations = self.value_portfolio(positions)?;
        Ok(valuations.iter().map(|v| v.value_usd).sum())
    }
}   