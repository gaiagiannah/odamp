//! Risk overlay: blocks rebalancing when risk conditions are unfavorable.
//!
//! Examples:
//! - Market volatility above threshold → block buys
//! - Geopolitical event affecting asset class → block that class
//! - Protocol risk rating below threshold → block DeFi positions
//! - Sanctions change affecting jurisdiction → block cross-border

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskOverlayResult {
    pub approved: bool,
    pub blocked_reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub risk_score: f64, // 0-100, higher = more risk
}

/// Evaluates risk conditions before allowing a rebalance.
pub struct RiskOverlay {
    max_volatility_pct: f64,
    max_protocol_risk_score: f64,
    blocked_jurisdictions: Vec<String>,
}

impl RiskOverlay {
    pub fn new(max_volatility: f64, max_protocol_risk: f64) -> Self {
        Self {
            max_volatility_pct: max_volatility,
            max_protocol_risk_score: max_protocol_risk,
            blocked_jurisdictions: vec![],
        }
    }

    pub fn evaluate(
        &self,
        current_volatility: f64,
        protocol_risk_scores: &std::collections::HashMap<String, f64>,
        target_jurisdiction: Option<&str>,
    ) -> RiskOverlayResult {
        let mut blocked = vec![];
        let mut warnings = vec![];
        let mut risk_score = 0.0;

        // Volatility check
        if current_volatility > self.max_volatility_pct {
            blocked.push(format!(
                "Market volatility {:.1}% exceeds threshold {:.1}%",
                current_volatility, self.max_volatility_pct
            ));
            risk_score += 30.0;
        } else if current_volatility > self.max_volatility_pct * 0.8 {
            warnings.push(format!("Elevated volatility: {:.1}%", current_volatility));
            risk_score += 10.0;
        }

        // Protocol risk check
        for (protocol, score) in protocol_risk_scores {
            if *score > self.max_protocol_risk_score {
                blocked.push(format!(
                    "Protocol {} risk score {:.0} exceeds threshold {:.0}",
                    protocol, score, self.max_protocol_risk_score
                ));
                risk_score += 20.0;
            }
        }

        // Jurisdiction check
        if let Some(jur) = target_jurisdiction {
            if self.blocked_jurisdictions.contains(&jur.to_string()) {
                blocked.push(format!("Jurisdiction {} is blocked", jur));
                risk_score += 50.0;
            }
        }

        RiskOverlayResult {
            approved: blocked.is_empty(),
            blocked_reasons: blocked,
            warnings,
            risk_score: risk_score.min(100.0),
        }
    }
}

impl Default for RiskOverlay {
    fn default() -> Self {
        Self::new(80.0, 70.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_overlay_blocks_high_vol() {
        let overlay = RiskOverlay::new(50.0, 70.0);
        let mut proto_risk = std::collections::HashMap::new();
        proto_risk.insert("aave-v3".to_string(), 20.0);

        let result = overlay.evaluate(75.0, &proto_risk, None);
        assert!(!result.approved);
        assert!(result.blocked_reasons.len() >= 1);
    }

    #[test]
    fn test_risk_overlay_approves_normal() {
        let overlay = RiskOverlay::default();
        let mut proto_risk = std::collections::HashMap::new();
        proto_risk.insert("aave-v3".to_string(), 20.0);

        let result = overlay.evaluate(35.0, &proto_risk, None);
        assert!(result.approved);
    }
}   