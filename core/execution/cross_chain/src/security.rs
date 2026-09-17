//! Bridge security monitoring: exploit alerts, TVL tracking, audit status.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Security status for a bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeSecurityStatus {
    pub bridge_id: String,
    pub security_score: f64,
    pub last_audit: Option<DateTime<Utc>>,
    pub auditor: Option<String>,
    pub tvl_usd: f64,
    pub tvl_change_24h_pct: f64,
    pub active_exploit: bool,
    pub exploit_details: Option<String>,
    pub historical_exploits: Vec<HistoricalExploit>,
    pub is_recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalExploit {
    pub date: DateTime<Utc>,
    pub amount_usd: f64,
    pub description: String,
    pub resolved: bool,
}

/// Monitors bridge security in real-time.
pub struct BridgeSecurityMonitor {
    statuses: HashMap<String, BridgeSecurityStatus>,
    min_security_score: f64,
    max_tvl_drop_pct: f64,
}

impl BridgeSecurityMonitor {
    pub fn new(min_security_score: f64, max_tvl_drop_pct: f64) -> Self {
        Self {
            statuses: HashMap::new(),
            min_security_score,
            max_tvl_drop_pct,
        }
    }

    pub fn update(&mut self, status: BridgeSecurityStatus) {
        self.statuses.insert(status.bridge_id.clone(), status);
    }

    /// Returns bridges that are safe to use.
    pub fn safe_bridges(&self) -> Vec<&BridgeSecurityStatus> {
        self.statuses
            .values()
            .filter(|s| {
                !s.active_exploit
                    && s.security_score >= self.min_security_score
                    && s.tvl_change_24h_pct > -self.max_tvl_drop_pct
            })
            .collect()
    }

    /// Checks if a specific bridge is safe for a given amount.
    pub fn is_safe(&self, bridge_id: &str, amount_usd: f64) -> Result<(), String> {
        let status = self.statuses.get(bridge_id)
            .ok_or_else(|| format!("Bridge {} not found", bridge_id))?;

        if status.active_exploit {
            return Err(format!(
                "Bridge {} has ACTIVE EXPLOIT: {}",
                bridge_id,
                status.exploit_details.as_deref().unwrap_or("unknown")
            ));
        }

        if status.security_score < self.min_security_score {
            return Err(format!(
                "Bridge {} security score {:.0} below threshold {:.0}",
                bridge_id, status.security_score, self.min_security_score
            ));
        }

        if amount_usd > status.tvl_usd * 0.1 {
            return Err(format!(
                "Transfer amount ${:.0} exceeds 10% of bridge TVL ${:.0}",
                amount_usd, status.tvl_usd
            ));
        }

        Ok(())
    }
}

impl Default for BridgeSecurityMonitor {
    fn default() -> Self {
        Self::new(80.0, 30.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_security_check() {
        let mut monitor = BridgeSecurityMonitor::default();

        monitor.update(BridgeSecurityStatus {
            bridge_id: "wormhole".into(),
            security_score: 85.0,
            last_audit: Some(Utc::now() - chrono::Duration::days(30)),
            auditor: Some("Trail of Bits".into()),
            tvl_usd: 1_800_000_000.0,
            tvl_change_24h_pct: -2.0,
            active_exploit: false,
            exploit_details: None,
            historical_exploits: vec![],
            is_recommended: true,
        });

        assert!(monitor.is_safe("wormhole", 5_000.0).is_ok());

        // Too large relative to TVL
        monitor.update(BridgeSecurityStatus {
            bridge_id: "small_bridge".into(),
            security_score: 90.0,
            last_audit: None,
            auditor: None,
            tvl_usd: 10_000.0,
            tvl_change_24h_pct: 0.0,
            active_exploit: false,
            exploit_details: None,
            historical_exploits: vec![],
            is_recommended: false,
        });

        assert!(monitor.is_safe("small_bridge", 5_000.0).is_err());
    }
}   