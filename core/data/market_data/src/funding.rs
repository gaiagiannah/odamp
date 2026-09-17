//! Funding rate tracking for perpetual futures.
//! Used by the Yield Optimizer agent to identify arbitrage opportunities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRate {
    pub symbol: String,
    pub venue: String,
    pub rate: f64,           // per 8h period (e.g., 0.0001 = 0.01%)
    pub annualized: f64,     // rate * 3 * 365
    pub next_funding_time: chrono::DateTime<chrono::Utc>,
    pub open_interest: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Tracks funding rates across venues for cross-venue arbitrage detection.
pub struct FundingTracker {
    rates: HashMap<String, Vec<FundingRate>>, // symbol → rates
}

impl FundingTracker {
    pub fn new() -> Self {
        Self { rates: HashMap::new() }
    }

    pub fn update(&mut self, rate: FundingRate) {
        self.rates.entry(rate.symbol.clone()).or_default().push(rate);
    }

    /// Finds funding rate arbitrage opportunities (long spot, short perp).
    pub fn find_arbitrage(&self, min_annualized: f64) -> Vec<FundingArb> {
        let mut opportunities = vec![];

        for (symbol, rates) in &self.rates {
            // Find venue with highest positive funding (best to short)
            let best_short = rates
                .iter()
                .filter(|r| r.annualized > min_annualized)
                .max_by(|a, b| a.annualized.partial_cmp(&b.annualized).unwrap());

            if let Some(short_venue) = best_short {
                opportunities.push(FundingArb {
                    symbol: symbol.clone(),
                    action: "long_spot_short_perp".into(),
                    venue: short_venue.venue.clone(),
                    annualized_return: short_venue.annualized,
                    open_interest: short_venue.open_interest,
                });
            }
        }

        opportunities
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingArb {
    pub symbol: String,
    pub action: String,
    pub venue: String,
    pub annualized_return: f64,
    pub open_interest: f64,
}

impl Default for FundingTracker {
    fn default() -> Self {
        Self::new()
    }
}   