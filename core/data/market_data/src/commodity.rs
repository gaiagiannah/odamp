//! Commodity price tracking: oil, gas, metals, rare earths, agricultural.
//! Sources: Chainlink oracles, LME, COMEX, satellite IoT verification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Commodity classification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommodityClass {
    Energy,        // Oil, gas, electricity
    PreciousMetals, // Gold, silver, platinum
    BaseMetals,    // Copper, aluminum, zinc
    RareEarths,    // Neodymium, dysprosium, lithium, cobalt
    Agricultural,  // Wheat, corn, soy, coffee
    Water,         // Water rights, credits
    Carbon,        // Carbon credits
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommodityQuote {
    pub symbol: String,
    pub name: String,
    pub class: CommodityClass,
    pub price_usd: f64,
    pub unit: String,          // "barrel", "oz", "tonne", "gallon"
    pub exchange: String,      // "LME", "COMEX", "ICE", "Chainlink"
    pub change_24h_pct: f64,
    pub change_7d_pct: f64,
    pub change_30d_pct: f64,
    /// Satellite/IoT verification status (for tokenized physical assets)
    pub physical_verified: bool,
    pub verification_source: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Tracks commodity prices and physical verification status.
pub struct CommodityTracker {
    quotes: HashMap<String, CommodityQuote>,
}

impl CommodityTracker {
    pub fn new() -> Self {
        Self { quotes: HashMap::new() }
    }

    pub fn update(&mut self, quote: CommodityQuote) {
        self.quotes.insert(quote.symbol.clone(), quote);
    }

    pub fn get(&self, symbol: &str) -> Option<&CommodityQuote> {
        self.quotes.get(symbol)
    }

    /// Returns all commodities in a given class.
    pub fn by_class(&self, class: &CommodityClass) -> Vec<&CommodityQuote> {
        self.quotes
            .values()
            .filter(|q| &q.class == class)
            .collect()
    }

    /// Returns commodities with active physical verification.
    pub fn physically_verified(&self) -> Vec<&CommodityQuote> {
        self.quotes.values().filter(|q| q.physical_verified).collect()
    }
}

impl Default for CommodityTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commodity_tracking() {
        let mut tracker = CommodityTracker::new();
        tracker.update(CommodityQuote {
            symbol: "WTI".into(),
            name: "West Texas Intermediate".into(),
            class: CommodityClass::Energy,
            price_usd: 78.50,
            unit: "barrel".into(),
            exchange: "COMEX".into(),
            change_24h_pct: 1.2,
            change_7d_pct: -0.5,
            change_30d_pct: 3.1,
            physical_verified: true,
            verification_source: Some("satellite_iot_tank7_rotterdam".into()),
            timestamp: chrono::Utc::now(),
        });

        let energy = tracker.by_class(&CommodityClass::Energy);
        assert_eq!(energy.len(), 1);
        assert!(energy[0].physical_verified);
    }
}   