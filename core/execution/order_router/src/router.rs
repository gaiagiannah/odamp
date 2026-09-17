//! The main OrderRouter: takes an order request, scores venues,
//! selects the best route, and produces a routing decision.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use crate::venue::{Venue, VenueQuote, Side};
use crate::scoring::{VenueScorer, ScoredVenue};
use crate::splitting::{OrderSplitter, SplitStrategy, ChildOrder};
use crate::error::RouterError;

/// An incoming order request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: Side,
    pub amount: f64,
    pub limit_price: Option<f64>,
    pub max_slippage_bps: f64,
    pub strategy: SplitStrategy,
    pub prefer_venues: Option<Vec<String>>,
    pub exclude_venues: Option<Vec<String>>,
}

/// The routing decision: where and how to execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub order_id: uuid::Uuid,
    pub symbol: String,
    pub side: Side,
    pub total_amount: f64,
    pub child_orders: Vec<ChildOrder>,
    pub selected_venues: Vec<String>,
    pub expected_cost_usd: f64,
    pub expected_slippage_bps: f64,
    pub expected_fee_usd: f64,
    pub alternative_routes: Vec<ScoredVenue>,
    pub decision_timestamp: chrono::DateTime<chrono::Utc>,
}

/// The smart order router.
pub struct OrderRouter {
    venues: Vec<Venue>,
    scorer: VenueScorer,
    splitter: OrderSplitter,
    max_slippage_bps: f64,
    execution_history: RwLock<Vec<RoutingDecision>>,
}

impl OrderRouter {
    pub fn new(venues: Vec<Venue>, max_slippage_bps: f64) -> Self {
        Self {
            venues,
            scorer: VenueScorer::default(),
            splitter: OrderSplitter::default(),
            max_slippage_bps,
            execution_history: RwLock::new(vec![]),
        }
    }

    /// Routes an order: scores venues, selects best, produces execution plan.
    pub fn route(
        &self,
        request: &OrderRequest,
        available_quotes: &[VenueQuote],
    ) -> Result<RoutingDecision, RouterError> {
        if available_quotes.is_empty() {
            return Err(RouterError::NoVenues(request.symbol.clone()));
        }

        // Filter by preferences
        let filtered: Vec<&VenueQuote> = available_quotes
            .iter()
            .filter(|q| {
                let venue = self.venues.iter().find(|v| v.id == q.venue_id);
                if venue.map(|v| !v.is_available).unwrap_or(true) {
                    return false;
                }
                if let Some(exclude) = &request.exclude_venues {
                    if exclude.contains(&q.venue_id) {
                        return false;
                    }
                }
                true
            })
            .collect();

        if filtered.is_empty() {
            return Err(RouterError::NoVenues(request.symbol.clone()));
        }

        // Score venues
        let owned_quotes: Vec<VenueQuote> = filtered.iter().map(|q| (*q).clone()).collect();
        let scored = self.scorer.score(&owned_quotes, &self.venues, request.amount);

                if scored.is_empty() {
            return Err(RouterError::NoVenues(request.symbol.clone()));
        }

        // Select best venue(s) based on strategy
        let order_id = uuid::Uuid::new_v4();

        // For single/parallel strategies, use top venues
        let selected: Vec<&ScoredVenue> = match &request.strategy {
            SplitStrategy::Single | SplitStrategy::Twap { .. } => scored.iter().take(1).collect(),
            SplitStrategy::Parallel | SplitStrategy::Vwap { .. } => {
                let n = (request.amount * scored[0].quote.price / 50_000.0).ceil() as usize;
                scored.iter().take(n.max(1).min(scored.len())).collect()
            }
        };

        // Check slippage constraint
        let max_slippage = selected
            .iter()
            .map(|s| s.quote.estimated_slippage_bps)
            .fold(0.0_f64, f64::max);

        if max_slippage > request.max_slippage_bps {
            return Err(RouterError::SlippageExceeded {
                actual_bps: max_slippage,
                max_bps: request.max_slippage_bps,
            });
        }

        // Build child orders
        let quotes_for_split: Vec<VenueQuote> = selected.iter().map(|s| s.quote.clone()).collect();
        let child_orders = self.splitter.split(
            order_id,
            request.side.clone(),
            request.amount,
            quotes_for_split[0].price,
            &request.strategy,
            &quotes_for_split,
        );

        // Calculate expected costs
        let total_value = request.amount * quotes_for_split[0].price;
        let avg_fee_bps = selected.iter()
            .map(|s| s.quote.fee_bps)
            .sum::<f64>() / selected.len() as f64;
        let expected_fee = total_value * (avg_fee_bps / 10_000.0);
        let expected_slippage_cost = total_value * (max_slippage / 10_000.0);

        let decision = RoutingDecision {
            order_id,
            symbol: request.symbol.clone(),
            side: request.side.clone(),
            total_amount: request.amount,
            child_orders,
            selected_venues: selected.iter().map(|s| s.venue_id.clone()).collect(),
            expected_cost_usd: total_value + expected_fee + expected_slippage_cost,
            expected_slippage_bps: max_slippage,
            expected_fee_usd: expected_fee,
            alternative_routes: scored.iter().skip(selected.len()).take(3).cloned().collect(),
            decision_timestamp: chrono::Utc::now(),
        };

        // Record in history
        self.execution_history.write().push(decision.clone());

        Ok(decision)
    }

    /// Returns execution history for TCA analysis.
    pub fn history(&self) -> Vec<RoutingDecision> {
        self.execution_history.read().clone()
    }

    /// Calculates actual vs. expected slippage for TCA.
    pub fn tca_summary(&self) -> TcaSummary {
        let history = self.execution_history.read();
        if history.is_empty() {
            return TcaSummary::default();
        }

        let total_expected_slippage: f64 = history.iter().map(|h| h.expected_slippage_bps).sum();
        let avg_expected = total_expected_slippage / history.len() as f64;
        let total_fees: f64 = history.iter().map(|h| h.expected_fee_usd).sum();
        let total_value: f64 = history.iter().map(|h| h.total_amount).sum();

        TcaSummary {
            total_orders: history.len() as u32,
            avg_slippage_bps: avg_expected,
            total_fees_usd: total_fees,
            total_value_usd: total_value,
            effective_cost_bps: if total_value > 0.0 {
                (total_fees / total_value) * 10_000.0 + avg_expected
            } else { 0.0 },
        }
    }
}

/// Transaction Cost Analysis summary.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TcaSummary {
    pub total_orders: u32,
    pub avg_slippage_bps: f64,
    pub total_fees_usd: f64,
    pub total_value_usd: f64,
    pub effective_cost_bps: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_selects_best_venue() {
        let venues = vec![
            Venue::coinbase(),
            Venue::kraken(),
            Venue::binance(),
        ];

        let router = OrderRouter::new(venues, 50.0);

        let quotes = vec![
            VenueQuote {
                venue_id: "coinbase".into(),
                symbol: "BTC".into(),
                side: Side::Buy,
                amount: 1.0,
                price: 100_500.0,
                fee_bps: 120.0,
                estimated_slippage_bps: 10.0,
                liquidity_available: 5.0,
                execution_time_ms: 50,
                is_guaranteed: true,
                timestamp: chrono::Utc::now(),
            },
            VenueQuote {
                venue_id: "binance".into(),
                symbol: "BTC".into(),
                side: Side::Buy,
                amount: 1.0,
                price: 100_010.0,
                fee_bps: 10.0,
                estimated_slippage_bps: 3.0,
                liquidity_available: 50.0,
                execution_time_ms: 30,
                is_guaranteed: true,
                timestamp: chrono::Utc::now(),
            },
            VenueQuote {
                venue_id: "kraken".into(),
                symbol: "BTC".into(),
                side: Side::Buy,
                amount: 1.0,
                price: 100_200.0,
                fee_bps: 26.0,
                estimated_slippage_bps: 5.0,
                liquidity_available: 20.0,
                execution_time_ms: 45,
                is_guaranteed: true,
                timestamp: chrono::Utc::now(),
            },
        ];

        let request = OrderRequest {
            symbol: "BTC".into(),
            side: Side::Buy,
            amount: 1.0,
            limit_price: Some(101_000.0),
            max_slippage_bps: 50.0,
            strategy: SplitStrategy::Single,
            prefer_venues: None,
            exclude_venues: None,
        };

        let decision = router.route(&request, &quotes).unwrap();
        assert_eq!(decision.selected_venues[0], "binance");
        assert!(decision.expected_slippage_bps < 5.0);
    }

    #[test]
    fn test_router_rejects_high_slippage() {
        let venues = vec![Venue::coinbase()];
        let router = OrderRouter::new(venues, 5.0); // max 5 bps

        let quotes = vec![VenueQuote {
            venue_id: "coinbase".into(),
            symbol: "SMALLCOIN".into(),
            side: Side::Buy,
            amount: 1000.0,
            price: 0.001,
            fee_bps: 120.0,
            estimated_slippage_bps: 50.0, // way over limit
            liquidity_available: 500.0,
            execution_time_ms: 100,
            is_guaranteed: false,
            timestamp: chrono::Utc::now(),
        }];

        let request = OrderRequest {
            symbol: "SMALLCOIN".into(),
            side: Side::Buy,
            amount: 1000.0,
            limit_price: None,
            max_slippage_bps: 5.0,
            strategy: SplitStrategy::Single,
            prefer_venues: None,
            exclude_venues: None,
        };

        let result = router.route(&request, &quotes);
        assert!(matches!(result, Err(RouterError::SlippageExceeded { .. })));
    }
}   