//! Liquidity depth analysis: exit time estimation, slippage prediction,
//! and venue liquidity comparison.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Liquidity data for a single venue/pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityQuote {
    pub venue: String,
    pub symbol: String,
    pub bid_depth: Vec<(f64, f64)>, // (price, amount) levels
    pub ask_depth: Vec<(f64, f64)>,
    pub spread_bps: f64,
    pub volume_24h: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Estimated cost to exit a position of a given size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitEstimate {
    pub symbol: String,
    pub amount: f64,
    pub best_venue: String,
    pub estimated_slippage_bps: f64,
    pub estimated_time_secs: f64,
    pub venues_considered: usize,
    pub confidence: f64,
}

/// Calculates liquidity metrics and exit estimates.
pub struct LiquidityAnalyzer {
    quotes: HashMap<String, Vec<LiquidityQuote>>, // symbol → quotes
}

impl LiquidityAnalyzer {
    pub fn new() -> Self {
        Self { quotes: HashMap::new() }
    }

    pub fn update_quote(&mut self, quote: LiquidityQuote) {
        self.quotes
            .entry(quote.symbol.clone())
            .or_default()
            .push(quote);
    }

    /// Estimates the cost and time to liquidate a position.
    pub fn estimate_exit(&self, symbol: &str, amount: f64) -> Option<ExitEstimate> {
        let quotes = self.quotes.get(symbol)?;
        if quotes.is_empty() {
            return None;
        }

        let mut best_slippage = f64::MAX;
        let mut best_venue = String::new();
        let mut best_time = 0.0;

        for quote in quotes {
            let (slippage, time) = self.calculate_slippage(quote, amount);
            if slippage < best_slippage {
                best_slippage = slippage;
                best_venue = quote.venue.clone();
                best_time = time;
            }
        }

        Some(ExitEstimate {
            symbol: symbol.to_string(),
            amount,
            best_venue,
            estimated_slippage_bps: best_slippage,
            estimated_time_secs: best_time,
            venues_considered: quotes.len(),
            confidence: 0.8, // would be higher with more data
        })
    }

    fn calculate_slippage(&self, quote: &LiquidityQuote, amount: f64) -> (f64, f64) {
        let mut remaining = amount;
        let mut total_cost = 0.0;
        let mut mid_price = if !quote.ask_depth.is_empty() {
            quote.ask_depth[0].0
        } else {
            return (f64::MAX, 0.0);
        };

        for (price, depth) in &quote.ask_depth {
            if remaining <= 0.0 {
                break;
            }
            let filled = remaining.min(*depth);
            total_cost += filled * price;
            remaining -= filled;
        }

        if remaining > 0.0 {
            // Not enough liquidity in this venue
            return (f64::MAX, 0.0);
        }

        let avg_fill = total_cost / amount;
        let slippage_bps = ((avg_fill - mid_price) / mid_price) * 10_000.0;
        let time = (amount / (quote.volume_24h / 86400.0)).max(1.0);

        (slippage_bps.max(0.0), time)
    }
}

impl Default for LiquidityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}   