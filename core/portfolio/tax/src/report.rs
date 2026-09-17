//! Tax report generation: produces structured output for filing.

use serde::{Deserialize, Serialize};
use crate::jurisdiction::Jurisdiction;
use crate::gain_loss::{YearSummary, Disposition};
use crate::wash_sale::WashSaleFlag;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxReport {
    pub user_id: uuid::Uuid,
    pub tax_year: i32,
    pub jurisdiction: Jurisdiction,
    pub summary: YearSummary,
    pub dispositions: Vec<Disposition>,
    pub wash_sale_flags: Vec<WashSaleFlag>,
    pub total_tax_owed_usd: f64,
    pub estimated_tax_rate: f64,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl TaxReport {
    pub fn calculate_tax(&self) -> f64 {
        let short_tax = self.summary.short_term_gain_usd.max(0.0)
            * self.jurisdiction.capital_gains_rate(false);
        let long_tax = self.summary.long_term_gain_usd.max(0.0)
            * self.jurisdiction.capital_gains_rate(true);
        short_tax + long_tax
    }
}

/// Generates a tax report for a given year and jurisdiction.
pub fn generate_report(
    user_id: uuid::Uuid,
    year: i32,
    jurisdiction: &Jurisdiction,
    summary: YearSummary,
    dispositions: Vec<Disposition>,
    wash_flags: Vec<WashSaleFlag>,
) -> TaxReport {
    let report = TaxReport {
        user_id,
        tax_year: year,
        jurisdiction: jurisdiction.clone(),
        summary: summary.clone(),
        dispositions,
        wash_sale_flags: wash_flags.clone(),
        total_tax_owed_usd: 0.0,
        estimated_tax_rate: 0.0,
        generated_at: chrono::Utc::now(),
    };

    let tax = report.calculate_tax();
    let total_gain = report.summary.total_gain_usd;
    let rate = if total_gain > 0.0 { tax / total_gain } else { 0.0 };

    TaxReport {
        total_tax_owed_usd: tax,
        estimated_tax_rate: rate,
        ..report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gain_loss::YearSummary;

    #[test]
    fn test_report_generation() {
        let summary = YearSummary {
            tax_year: 2026,
            short_term_gain_usd: 5_000.0,
            long_term_gain_usd: 15_000.0,
            total_gain_usd: 20_000.0,
            disposition_count: 10,
            by_asset: vec![("BTC".into(), 12_000.0), ("ETH".into(), 8_000.0)],
        };

        let report = generate_report(
            uuid::Uuid::new_v4(),
            2026,
            &Jurisdiction::Us,
            summary,
            vec![],
            vec![],
        );

        // Short: 5000 * 0.37 = 1850
        // Long: 15000 * 0.15 = 2250
        // Total: 4100
        assert!((report.total_tax_owed_usd - 4_100.0).abs() < 1.0);
    }
}   