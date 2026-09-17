//! Venue scoring: ranks venues for a given order based on
//! price, fee, slippage, liquidity, reliability, and latency.

use serde::{Deserialize, Serialize};
use crate::venue::{Venue, VenueQuote, Side};

/// Weighted scoring configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    pub price_weight: f64,        // how much raw price matters
    pub fee_weight: f64,          // how much fees matter
    pub slippage_weight: f64,     // how much slippage matters
    pub liquidity_weight: f64,    // how much available depth matters
    pub reliability_weight: f64,  // how much venue uptime matters
    pub speed_weight: f64,        // how much latency matters
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            price_weight: 0.30,
            fee_weight: 0.20,
            slippage_weight: 0.25,
            liquidity_weight: 0.10,
            reliability_weight: 0.10,
            speed_weight: 0.05,
        }
    }
}

/// A scored venue for routing decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredVenue {
    pub venue_id: String,
    pub score: f64,
    pub quote: VenueQuote,
    pub score_breakdown: ScoreBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub price_score: f64,
    pub fee_score: f64,
    pub slippage_score: f64,
    pub liquidity_score: f64,
    pub reliability_score: f64,
    pub speed_score: f64,
}

/// Scores and ranks venues for a given order.
pub struct VenueScorer {
    weights: ScoringWeights,
}

impl VenueScorer {
    pub fn new(weights: ScoringWeights) -> Self {
        Self { weights }
    }

    /// Scores all quotes and returns them ranked (best first).
    pub fn score(&self, quotes: &[VenueQuote], venues: &[Venue], order_amount: f64) -> Vec<ScoredVenue> {
        let mut scored: Vec<ScoredVenue> = Vec::new();

        // Find best/worst for normalization
        let best_price = quotes.iter().map(|q| q.price).fold(f64::MAX, f64::min);
        let worst_price = quotes.iter().map(|q| q.price).fold(f64::MIN, f64::max);
        let max_slippage = quotes.iter().map(|q| q.estimated_slippage_bps).fold(0.0_f64, f64::max);
        let max_liquidity = quotes.iter().map(|q| q.liquidity_available).fold(0.0_f64, f64::max);

        for quote in quotes {
            let venue = venues.iter().find(|v| v.id == quote.venue_id);
            let reliability = venue.map(|v| v.reliability_score).unwrap_or(50.0);
            let latency = venue.map(|v| v.latency_ms).unwrap_or(1000);

            let price_score = if worst_price > best_price {
                0.0
            } else {
                ((worst_price - quote.price) / (worst_price - best_price)) * 100.0
            };

            let fee_score = 100.0 - (quote.fee_bps / 10.0); // 10bps = 100, 100bps = 0

            let slippage_score = if max_slippage > 0.0 {
                (1.0 - quote.estimated_slippage_bps / max_slippage) * 100.0
            } else { 100.0 };

            let liquidity_score = if max_liquidity > 0.0 {
                (quote.liquidity_available / max_liquidity).min(1.0) * 100.0
            } else { 0.0 };

            let reliability_score = reliability;
            let speed_score = (1.0 - (latency as f64 / 5000.0)).clamp(0.0, 1.0) * 100.0;

            let total = (
                price_score * self.weights.price_weight
                + fee_score * self.weights.fee_weight
                + slippage_score * self.weights.slippage_weight
                + liquidity_score * self.weights.liquidity_weight
                + reliability_score * self.weights.reliability_weight
                + speed_score * self.weights.speed_weight
            );

            scored.push(ScoredVenue {
                venue_id: quote.venue_id.clone(),
                score: total,
                quote: quote.clone(),
                score_breakdown: ScoreBreakdown {
                    price_score,
                    fee_score: fee_score.max(0.0),
                    slippage_score,
                    liquidity_score,
                    reliability_score,
                    speed_score,
                },
            });
        }

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        scored
    }
}

impl Default for VenueScorer {
    fn default() -> Self {
        Self::new(ScoringWeights::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoring_ranks_best_first() {
        let scorer = VenueScorer::default();

        let quotes = vec![
            VenueQuote {
                venue_id: "expensive".into(),
                symbol: "BTC".into(),
                side: Side::Buy,
                amount: 1.0,
                price: 100_500.0,
                fee_bps: 200.0,
                estimated_slippage_bps: 20.0,
                liquidity_available: 5.0,
                execution_time_ms: 100,
                is_guaranteed: true,
                timestamp: chrono::Utc::now(),
            },
            VenueQuote {
                venue_id: "cheap".into(),
                symbol: "BTC".into(),
                side: Side::Buy,
                amount: 1.0,
                price: 100_010.0,
                fee_bps: 10.0,
                estimated_slippage_bps: 3.0,
                liquidity_available: 10.0,
                execution_time_ms: 50,
                is_guaranteed: true,
                timestamp: chrono::Utc::now(),
            },
        ];

        let venues = vec![
            Venue { id: "expensive".into(), name: "Expensive".into(), venue_type: crate::venue::VenueType::Cex, chain: None, base_fee_bps: 200.0, max_order_usd: 1_000_000.0, min_order_usd: 1.0, is_available: true, latency_ms: 100, reliability_score: 95.0 },
            Venue { id: "cheap".into(), name: "Cheap".into(), venue_type: crate::venue::VenueType::Cex, chain: None, base_fee_bps: 10.0, max_order_usd: 1_000_000.0, min_order_usd: 1.0, is_available: true, latency_ms: 50, reliability_score: 99.9 },
        ];

        let scored = scorer.score(&quotes, &venues, 1.0);
        assert_eq!(scored[0].venue_id, "cheap");
        assert!(scored[0].score > scored[1].score);
    }
}   