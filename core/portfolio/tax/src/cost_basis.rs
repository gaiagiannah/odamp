//! Cost basis tracking using FIFO, LIFO, or Specific Identification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::error::TaxError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CostBasisMethod {
    Fifo,
    Lifo,
    SpecificId,
    AverageCost,
}

/// A lot: a batch of the same asset acquired at the same time/price.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lot {
    pub asset_symbol: String,
    pub amount: f64,
    pub cost_per_unit: f64,
    pub total_cost: f64,
    pub acquired_at: DateTime<Utc>,
    pub source: String, // "exchange", "defi", "airdrop", "staking", "gift"
    pub tx_hash: Option<String>,
}

impl Lot {
    pub fn new(symbol: &str, amount: f64, cost_per_unit: f64, acquired_at: DateTime<Utc>, source: &str) -> Self {
        Self {
            asset_symbol: symbol.to_string(),
            amount,
            cost_per_unit,
            total_cost: amount * cost_per_unit,
            acquired_at,
            source: source.to_string(),
            tx_hash: None,
        }
    }
}

/// Tracks cost basis lots per asset.
pub struct CostBasisTracker {
    method: CostBasisMethod,
    lots: HashMap<String, Vec<Lot>>, // asset_symbol → lots (ordered by time)
}

impl CostBasisTracker {
    pub fn new(method: CostBasisMethod) -> Self {
        Self { method, lots: HashMap::new() }
    }

    /// Records an acquisition (buy, airdrop, staking reward, etc.).
    pub fn acquire(&mut self, lot: Lot) {
        self.lots.entry(lot.asset_symbol.clone()).or_default().push(lot);
    }

    /// Records a disposal (sell, swap, spend) and returns the cost basis used.
    pub fn dispose(&mut self, symbol: &str, amount: f64, disposed_at: DateTime<Utc>) -> Result<f64, TaxError> {
        let lots = self.lots.get_mut(symbol)
            .ok_or_else(|| TaxError::NoCostBasis(symbol.to_string()))?;

        if lots.is_empty() {
            return Err(TaxError::NoCostBasis(symbol.to_string()));
        }

        let total_available: f64 = lots.iter().map(|l| l.amount).sum();
        if total_available < amount - f64::EPSILON {
            return Err(TaxError::InsufficientHistory);
        }

        let mut cost_basis = 0.0;
        let mut remaining = amount;

        match self.method {
            CostBasisMethod::Fifo => {
                while remaining > f64::EPSILON && !lots.is_empty() {
                    let lot = &mut lots[0];
                    let used = remaining.min(lot.amount);
                    cost_basis += used * lot.cost_per_unit;
                    lot.amount -= used;
                    remaining -= used;
                    if lot.amount < f64::EPSILON {
                        lots.remove(0);
                    }
                }
            }
            CostBasisMethod::Lifo => {
                while remaining > f64::EPSILON && !lots.is_empty() {
                    let idx = lots.len() - 1;
                    let lot = &mut lots[idx];
                    let used = remaining.min(lot.amount);
                    cost_basis += used * lot.cost_per_unit;
                    lot.amount -= used;
                    remaining -= used;
                    if lot.amount < f64::EPSILON {
                        lots.pop();
                    }
                }
            }
            CostBasisMethod::AverageCost => {
                let total_cost: f64 = lots.iter().map(|l| l.total_cost).sum();
                let total_amount: f64 = lots.iter().map(|l| l.amount).sum();
                let avg = total_cost / total_amount;
                cost_basis = amount * avg;
                // Reduce lots proportionally
                let ratio = amount / total_amount;
                for lot in lots.iter_mut() {
                    lot.amount *= (1.0 - ratio);
                    lot.total_cost *= (1.0 - ratio);
                }
                lots.retain(|l| l.amount > f64::EPSILON);
            }
            CostBasisMethod::SpecificId => {
                // Caller must specify which lot; simplified here to FIFO
                return Err(TaxError::ConflictingMethods);
            }
        }

        Ok(cost_basis)
    }

    /// Returns current holdings (unrealized) for a symbol.
    pub fn holdings(&self, symbol: &str) -> (f64, f64) {
        // Returns (amount, total_cost)
        match self.lots.get(symbol) {
            None => (0.0, 0.0),
            Some(lots) => {
                let amount: f64 = lots.iter().map(|l| l.amount).sum();
                let cost: f64 = lots.iter().map(|l| l.total_cost).sum();
                (amount, cost)
            }
        }
    }

    pub fn method(&self) -> &CostBasisMethod {
        &self.method
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fifo_cost_basis() {
        let mut tracker = CostBasisTracker::new(CostBasisMethod::Fifo);
        let now = Utc::now();

        // Buy 10 BTC at $50,000
        tracker.acquire(Lot::new("BTC", 10.0, 50_000.0, now, "exchange"));
        // Buy 5 BTC at $60,000 (later)
        tracker.acquire(Lot::new("BTC", 5.0, 60_000.0, now + chrono::Duration::days(30), "exchange"));

        // Sell 8 BTC → FIFO uses first lot ($50K)
        let cost = tracker.dispose("BTC", 8.0, now + chrono::Duration::days(60)).unwrap();
        assert_eq!(cost, 8.0 * 50_000.0); // $400,000

        // Remaining: 2 BTC @ $50K + 5 BTC @ $60K
        let (amount, total_cost) = tracker.holdings("BTC");
        assert!((amount - 7.0).abs() < f64::EPSILON);
        assert!((total_cost - (2.0 * 50_000.0 + 5.0 * 60_000.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_average_cost() {
        let mut tracker = CostBasisTracker::new(CostBasisMethod::AverageCost);
        let now = Utc::now();

        tracker.acquire(Lot::new("ETH", 10.0, 2_000.0, now, "exchange"));
        tracker.acquire(Lot::new("ETH", 10.0, 3_000.0, now + chrono::Duration::days(10), "exchange"));

        // Average = (20000 + 30000) / 20 = 2500
        let cost = tracker.dispose("ETH", 5.0, now + chrono::Duration::days(20)).unwrap();
        assert!((cost - 5.0 * 2_500.0).abs() < 0.01);
    }
}   