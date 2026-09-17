//! Capital gain/loss calculation.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::error::TaxError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disposition {
    pub asset_symbol: String,
    pub amount: f64,
    pub proceeds_usd: f64,
    pub cost_basis_usd: f64,
    pub gain_loss_usd: f64,
    pub holding_period_days: u64,
    pub is_long_term: bool,
    pub disposed_at: DateTime<Utc>,
    pub event_type: String, // "sale", "swap", "spend", "nft_sale"
}

impl Disposition {
    pub fn gain_loss_pct(&self) -> f64 {
        if self.cost_basis_usd > 0.0 {
            (self.gain_loss_usd / self.cost_basis_usd) * 100.0
        } else { 0.0 }
    }
}

/// Calculates gains and losses for a tax year.
pub struct GainLossCalculator {
    dispositions: Vec<Disposition>,
}

impl GainLossCalculator {
    pub fn new() -> Self {
        Self { dispositions: vec![] }
    }

    pub fn record(&mut self, d: Disposition) {
        self.dispositions.push(d);
    }

    /// Returns summary for a tax year.
    pub fn yearly_summary(&self, year: i32) -> YearSummary {
        let year_dispositions: Vec<&Disposition> = self
            .dispositions
            .iter()
            .filter(|d| d.disposed_at.year() as i32 == year)
            .collect();

        let short_term: f64 = year_dispositions
            .iter()
            .filter(|d| !d.is_long_term)
            .map(|d| d.gain_loss_usd)
            .sum();

        let long_term: f64 = year_dispositions
            .iter()
            .filter(|d| d.is_long_term)
            .map(|d| d.gain_loss_usd)
            .sum();

        let total = short_term + long_term;

        YearSummary {
            tax_year: year,
            short_term_gain_usd: short_term,
            long_term_gain_usd: long_term,
            total_gain_usd: total,
            disposition_count: year_dispositions.len(),
            by_asset: self.group_by_asset(&year_dispositions),
        }
    }

    fn group_by_asset(&self, dispositions: &[&Disposition]) -> Vec<(String, f64)> {
        let mut map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for d in dispositions {
            *map.entry(d.asset_symbol.clone()).or_insert(0.0) += d.gain_loss_usd;
        }
        map.into_iter().collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearSummary {
    pub tax_year: i32,
    pub short_term_gain_usd: f64,
    pub long_term_gain_usd: f64,
    pub total_gain_usd: f64,
    pub disposition_count: usize,
    pub by_asset: Vec<(String, f64)>,
}

impl Default for GainLossCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yearly_summary() {
        let mut calc = GainLossCalculator::new();
        let now = Utc::now();

        calc.record(Disposition {
            asset_symbol: "BTC".into(),
            amount: 1.0,
            proceeds_usd: 70_000.0,
            cost_basis_usd: 50_000.0,
            gain_loss_usd: 20_000.0,
            holding_period_days: 400,
            is_long_term: true,
            disposed_at: now,
            event_type: "sale".into(),
        });

        calc.record(Disposition {
            asset_symbol: "ETH".into(),
            amount: 5.0,
            proceeds_usd: 12_000.0,
            cost_basis_usd: 15_000.0,
            gain_loss_usd: -3_000.0,
            holding_period_days: 30,
            is_long_term: false,
            disposed_at: now,
            event_type: "sale".into(),
        });

        let summary = calc.yearly_summary(now.year() as i32);
        assert_eq!(summary.long_term_gain_usd, 20_000.0);
        assert_eq!(summary.short_term_gain_usd, -3_000.0);
        assert_eq!(summary.total_gain_usd, 17_000.0);
        assert_eq!(summary.disposition_count, 2);
    }
}   