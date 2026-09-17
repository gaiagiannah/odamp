//! Order splitting: breaks large orders into smaller child orders
//! to minimize market impact.

use serde::{Deserialize, Serialize};
use crate::venue::{VenueQuote, Side};

/// Splitting strategy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SplitStrategy {
    /// Volume-Weighted Average Price: spread over time proportional to volume
    Vwap { duration_mins: u32 },
    /// Time-Weighted Average Price: equal splits at equal intervals
    Twap { interval_secs: u32 },
    /// Split across venues simultaneously (no time spreading)
    Parallel,
    /// Single order (no splitting)
    Single,
}

/// A child order resulting from a split.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildOrder {
    pub parent_order_id: uuid::Uuid,
    pub venue_id: String,
    pub side: Side,
    pub amount: f64,
    pub limit_price: Option<f64>,
    pub execute_at: Option<chrono::DateTime<chrono::Utc>>,
    pub sequence: u32,
}

/// Splits a parent order into child orders.
pub struct OrderSplitter {
    max_child_order_usd: f64,
    min_child_order_usd: f64,
}

impl OrderSplitter {
    pub fn new(max_child: f64, min_child: f64) -> Self {
        Self { max_child_order_usd: max_child, min_child_order_usd: min_child }
    }

    /// Splits an order according to the strategy.
    pub fn split(
        &self,
        order_id: uuid::Uuid,
        side: Side,
        total_amount: f64,
        price: f64,
        strategy: &SplitStrategy,
        available_venues: &[VenueQuote],
    ) -> Vec<ChildOrder> {
        match strategy {
            SplitStrategy::Single => vec![ChildOrder {
                parent_order_id: order_id,
                venue_id: available_venues.first().map(|q| q.venue_id.clone()).unwrap_or_default(),
                side,
                amount: total_amount,
                limit_price: Some(price),
                execute_at: None,
                sequence: 0,
            }],

            SplitStrategy::Parallel => {
                let n = available_venues.len().max(1);
                let per_venue = total_amount / n as f64;
                available_venues
                    .iter()
                    .enumerate()
                    .map(|(i, q)| ChildOrder {
                        parent_order_id: order_id,
                        venue_id: q.venue_id.clone(),
                        side,
                        amount: per_venue.min(self.max_child_order_usd),
                        limit_price: Some(q.price),
                        execute_at: None,
                        sequence: i as u32,
                    })
                    .collect()
            }

            SplitStrategy::Twap { interval_secs } => {
                let order_value = total_amount * price;
                let n_splits = ((order_value / self.max_child_order_usd).ceil() as u32).max(1);
                let per_split = total_amount / n_splits as f64;
                let now = chrono::Utc::now();

                (0..n_splits)
                    .map(|i| ChildOrder {
                        parent_order_id: order_id,
                        venue_id: available_venues.first().map(|q| q.venue_id.clone()).unwrap_or_default(),
                        side,
                        amount: per_split.max(self.min_child_order_usd),
                        limit_price: Some(price),
                        execute_at: Some(now + chrono::Duration::seconds((i as i64 * *interval_secs as i64))),
                        sequence: i,
                    })
                    .collect()
            }

            SplitStrategy::Vwap { duration_mins } => {
                // VWAP: weight splits by expected volume at each interval
                // Simplified: equal time intervals with slight randomization
                let n_splits = 10u32;
                let interval = (*duration_mins * 60) / n_splits as u32;
                let per_split = total_amount / n_splits as f64;
                let now = chrono::Utc::now();

                (0..n_splits)
                    .map(|i| {
                        // Slight randomization to avoid predictable patterns
                        let jitter = rand::random::<f64>() * 0.2 + 0.9;
                        ChildOrder {
                            parent_order_id: order_id,
                            venue_id: available_venues.first().map(|q| q.venue_id.clone()).unwrap_or_default(),
                            side,
                            amount: (per_split * jitter).max(self.min_child_order_usd),
                            limit_price: None, // market
                            execute_at: Some(now + chrono::Duration::seconds(i as i64 * interval as i64)),
                            sequence: i,
                        }
                    })
                    .collect()
            }
        }
    }
}

impl Default for OrderSplitter {
    fn default() -> Self {
        Self::new(50_000.0, 100.0) // max $50K per child, min $100
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::venue::Side;

    #[test]
    fn test_twap_splitting() {
        let splitter = OrderSplitter::new(10_000.0, 100.0);
        let quotes = vec![VenueQuote {
            venue_id: "test".into(),
            symbol: "BTC".into(),
            side: Side::Buy,
            amount: 1.0,
            price: 100_000.0,
            fee_bps: 10.0,
            estimated_slippage_bps: 1.0,
            liquidity_available: 100.0,
            execution_time_ms: 50,
            is_guaranteed: true,
            timestamp: chrono::Utc::now(),
        }];

        // $200K order → 20 splits of $10K
        let children = splitter.split(
            uuid::Uuid::new_v4(),
            Side::Buy,
            2.0, // 2 BTC = $200K
            100_000.0,
            &SplitStrategy::Twap { interval_secs: 60 },
            &quotes,
        );

        assert_eq!(children.len(), 20);
        assert!(children.iter().all(|c| c.execute_at.is_some()));
        // Each child should be ~$10K
        assert!((children[0].amount * 100_000.0 - 10_000.0).abs() < 1.0);
    }

    #[test]
    fn test_parallel_splitting() {
        let splitter = OrderSplitter::default();
        let quotes = vec![
            VenueQuote { venue_id: "a".into(), symbol: "ETH".into(), side: Side::Sell, amount: 10.0, price: 3000.0, fee_bps: 10.0, estimated_slippage_bps: 1.0, liquidity_available: 50.0, execution_time_ms: 50, is_guaranteed: true, timestamp: chrono::Utc::now() },
            VenueQuote { venue_id: "b".into(), symbol: "ETH".into(), side: Side::Sell, amount: 10.0, price: 3001.0, fee_bps: 12.0, estimated_slippage_bps: 2.0, liquidity_available: 30.0, execution_time_ms: 80, is_guaranteed: true, timestamp: chrono::Utc::now() },
        ];

        let children = splitter.split(
            uuid::Uuid::new_v4(),
            Side::Sell,
            20.0, // 20 ETH split across 2 venues
            3000.0,
            &SplitStrategy::Parallel,
            &quotes,
        );

        assert_eq!(children.len(), 2);
        assert_eq!(children[0].venue_id, "a");
        assert_eq!(children[1].venue_id, "b");
        assert!((children[0].amount - 10.0).abs() < 0.01);
    }
}   