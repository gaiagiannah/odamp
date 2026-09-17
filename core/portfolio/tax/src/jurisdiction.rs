//! Jurisdiction-specific tax rules.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Jurisdiction {
    Us,
    Eu,
    Uk,
    Singapore,
    Japan,
    India,
    ElSalvador,
    Other,
}

impl Jurisdiction {
    pub fn code(&self) -> &'static str {
        match self {
            Jurisdiction::Us => "US",
            Jurisdiction::Eu => "EU",
            Jurisdiction::Uk => "UK",
            Jurisdiction::Singapore => "SG",
            Jurisdiction::Japan => "JP",
            Jurisdiction::India => "IN",
            Jurisdiction::ElSalvador => "SV",
            Jurisdiction::Other => "XX",
        }
    }

    /// Long-term holding period in days.
    pub fn long_term_threshold_days(&self) -> u64 {
        match self {
            Jurisdiction::Us => 365,
            Jurisdiction::Eu => 365, // varies by country; 365 as default
            Jurisdiction::Uk => 365,
            _ => 365,
        }
    }

    /// Whether wash sale rules apply.
    pub fn wash_sale_rules(&self) -> bool {
        matches!(self, Jurisdiction::Us)
    }

    /// Capital gains tax rate (simplified; real rates vary by income bracket).
    pub fn capital_gains_rate(&self, is_long_term: bool) -> f64 {
        match self {
            Jurisdiction::Us => {
                if is_long_term { 0.15 } else { 0.37 } // simplified top rates
            }
            Jurisdiction::Uk => {
                if is_long_term { 0.24 } else { 0.33 }
            }
            Jurisdiction::Eu => {
                if is_long_term { 0.19 } else { 0.26 } // Germany as example
            }
            Jurisdiction::India => 0.30, // flat 30%
            Jurisdiction::ElSalvador => 0.0, // crypto tax exempt
            _ => 0.25,
        }
    }

    /// Whether staking rewards are taxed as income at receipt.
    pub fn staking_taxed_at_receipt(&self) -> bool {
        matches!(self, Jurisdiction::Us | Jurisdiction::Uk | Jurisdiction::Eu)
    }

    /// Reporting form name.
    pub fn reporting_form(&self) -> &'static str {
        match self {
            Jurisdiction::Us => "1099-DA / Schedule D",
            Jurisdiction::Eu => "DAC8",
            Jurisdiction::Uk => "Self Assessment (SA100)",
            _ => "N/A",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jurisdiction_rules() {
        assert!(Jurisdiction::Us.wash_sale_rules());
        assert!(!Jurisdiction::Uk.wash_sale_rules());
        assert_eq!(Jurisdiction::Us.long_term_threshold_days(), 365);
        assert!(Jurisdiction::ElSalvador.capital_gains_rate(true) == 0.0);
    }
}   