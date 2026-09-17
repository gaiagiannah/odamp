//! Wash sale detection (U.S. IRS rules).
//!
//! A wash sale occurs when you sell a security at a loss and buy
//! "substantially identical" securities within 30 days before or after.
//! The loss is disallowed and added to the cost basis of the new purchase.
//!
//! For crypto: The IRS has not explicitly confirmed wash sale rules apply
//! to digital assets, but this engine flags them for user awareness.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WashSaleFlag {
    pub asset_symbol: String,
    pub sale_date: DateTime<Utc>,
    pub repurchase_date: DateTime<Utc>,
    pub days_apart: i64,
    pub disallowed_loss_usd: f64,
    pub adjusted_cost_basis_usd: f64,
    pub is_confirmed: bool, // true if within 30-day window
}

/// Detects potential wash sales.
pub struct WashSaleDetector {
    window_days: i64,
    transactions: Vec<WashSaleTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WashSaleTransaction {
    pub asset_symbol: String,
    pub is_purchase: bool,
    pub amount: f64,
    pub price_usd: f64,
    pub timestamp: DateTime<Utc>,
}

impl WashSaleDetector {
    pub fn new(window_days: i64) -> Self {
        Self { window_days, transactions: vec![] }
    }

    pub fn add_transaction(&mut self, tx: WashSaleTransaction) {
        self.transactions.push(tx);
    }

    /// Scans all transactions for wash sale patterns.
    pub fn detect(&self) -> Vec<WashSaleFlag> {
        let mut flags = vec![];

        // Group by asset
        let mut by_asset: HashMap<&str, Vec<&WashSaleTransaction>> = HashMap::new();
        for tx in &self.transactions {
            by_asset.entry(tx.asset_symbol.as_str()).or_default().push(tx);
        }

        for (symbol, txs) in &by_asset {
            let sells: Vec<&&WashSaleTransaction> = txs.iter().filter(|t| !t.is_purchase).collect();
            let buys: Vec<&&WashSaleTransaction> = txs.iter().filter(|t| t.is_purchase).collect();

            for sell in &sells {
                for buy in &buys {
                    let diff = (*buy).timestamp - (*sell).timestamp;
                    let days = diff.num_days();

                    if days.abs() <= self.window_days && days != 0 {
                        let loss = (*sell).amount * (*sell).price_usd; // simplified
                        flags.push(WashSaleFlag {
                            asset_symbol: symbol.to_string(),
                            sale_date: (*sell).timestamp,
                            repurchase_date: (*buy).timestamp,
                            days_apart: days,
                            disallowed_loss_usd: loss,
                            adjusted_cost_basis_usd: (*buy).amount * (*buy).price_usd + loss,
                            is_confirmed: true,
                        });
                    }
                }
            }
        }

        flags
    }
}

impl Default for WashSaleDetector {
    fn default() -> Self {
        Self::new(30)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wash_sale_detection() {
        let mut detector = WashSaleDetector::new(30);
        let now = Utc::now();

        detector.add_transaction(WashSaleTransaction {
            asset_symbol: "BTC".into(),
            is_purchase: false,
            amount: 1.0,
            price_usd: 50_000.0,
            timestamp: now,
        });

        detector.add_transaction(WashSaleTransaction {
            asset_symbol: "BTC".into(),
            is_purchase: true,
            amount: 1.0,
            price_usd: 48_000.0,
            timestamp: now + Duration::days(10),
        });

        let flags = detector.detect();
        assert_eq!(flags.len(), 1);
        assert!(flags[0].is_confirmed);
        assert_eq!(flags[0].days_apart, 10);
    }

    #[test]
    fn test_no_wash_sale_outside_window() {
        let mut detector = WashSaleDetector::new(30);
        let now = Utc::now();

        detector.add_transaction(WashSaleTransaction {
            asset_symbol: "BTC".into(),
            is_purchase: false,
            amount: 1.0,
            price_usd: 50_000.0,
            timestamp: now,
        });

        detector.add_transaction(WashSaleTransaction {
            asset_symbol: "BTC".into(),
            is_purchase: true,
            amount: 1.0,
            price_usd: 48_000.0,
            timestamp: now + Duration::days(45), // outside 30-day window
        });

        let flags = detector.detect();
        assert!(flags.is_empty());
    }
}   