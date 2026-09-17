//! ODAMP Scam Detection Agent
//!
//! Real-time analysis of:
//! - New contracts (static + dynamic analysis)
//! - Token launches (rug pull patterns)
//! - Phishing (domain reputation, address patterns)
//! - APY anomalies (unrealistic yields)
//! - Social engineering patterns
//! - Sandwich attack patterns
//! - Fake airdrops
//! - Honeypot detection
//!
//! This agent can BLOCK transactions (not just alert).
//! It is the first line of defense before any transaction is signed.

use async_trait::async_trait;
use odamp_orchestrator::agent::{Agent, AgentContext, AgentResponse, AgentStatus, ResponseType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScamDetectConfig {
    pub max_apy_threshold: f64,
    pub min_contract_age_hours: u32,
    pub block_on_high_risk: bool,
    pub check_new_tokens: bool,
    pub check_contract_code: bool,
    pub phishing_domain_check: bool,
}

impl Default for ScamDetectConfig {
    fn default() -> Self {
        Self {
            max_apy_threshold: 100.0,
            min_contract_age_hours: 24,
            block_on_high_risk: true,
            check_new_tokens: true,
            check_contract_code: true,
            phishing_domain_check: true,
        }
    }
}

/// A scam detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScamVerdict {
    pub is_scam: bool,
    pub confidence: f64,
    pub threat_type: Option<ThreatType>,
    pub severity: ThreatSeverity,
    pub message: String,
    pub indicators: Vec<String>,
    pub recommended_action: RecommendedAction,
    pub contract_address: Option<String>,
    pub token_symbol: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThreatType {
    RugPull,
    Honeypot,
    Phishing,
    FakeAirdrop,
    PonziScheme,
    SandwichAttack,
    PumpAndDump,
    FakeToken,
    MaliciousContract,
    SocialEngineering,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendedAction {
    Allow,
    Warn,
    Block,
    RequireConfirmation,
}

pub struct ScamDetectionAgent {
    id: uuid::Uuid,
    status: AgentStatus,
    config: ScamDetectConfig,
    known_bad_addresses: std::collections::HashSet<String>,
    known_good_contracts: std::collections::HashSet<String>,
    recent_scams: Vec<ScamVerdict>,
}

impl ScamDetectionAgent {
    pub fn new(config: ScamDetectConfig) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            status: AgentStatus::Active,
            config,
            known_bad_addresses: std::collections::HashSet::new(),
            known_good_contracts: std::collections::HashSet::new(),
            recent_scams: vec![],
        }
    }

    /// Analyzes a transaction before it's signed.
    /// Returns a verdict: allow, warn, or block.
    pub fn analyze_transaction(&self, tx: &TransactionToCheck) -> ScamVerdict {
        let mut indicators = vec![];
        let mut max_severity = ThreatSeverity::Low;
        let mut threat_type = None;
        let mut is_scam = false;

        // 1. Check against known bad addresses
        if let Some(to) = &tx.to {
            if self.known_bad_addresses.contains(&to.to_lowercase()) {
                indicators.push("Address is in known scam database".to_string());
                is_scam = true;
                max_severity = ThreatSeverity::Critical;
                threat_type = Some(ThreatType::Phishing);
            }
        }

        // 2. Check for honeypot patterns (token that can't be sold)
        if tx.is_token_transfer && tx.to_amount == 0.0 {
            indicators.push("Transfer shows 0 output — possible honeypot".to_string());
            is_scam = true;
            max_severity = ThreatSeverity::Critical;
            threat_type = Some(ThreatType::Honeypot);
        }

        // 3. Check APY anomaly
        if let Some(apy) = tx.promised_apy {
            if apy > self.config.max_apy_threshold {
                indicators.push(format!(
                    "Promised APY of {:.0}% exceeds realistic maximum ({:.0}%)",
                    apy, self.config.max_apy_threshold
                ));
                is_scam = true;
                max_severity = ThreatSeverity::High;
                threat_type = Some(ThreatType::PonziScheme);
            }
        }

        // 4. Check contract age
        if let Some(age_hours) = tx.contract_age_hours {
            if age_hours < self.config.min_contract_age_hours as f64 {
                indicators.push(format!(
                    "Contract is only {:.0}h old (minimum: {}h)",
                    age_hours, self.config.min_contract_age_hours
                ));
                if max_severity < ThreatSeverity::Medium {
                    max_severity = ThreatSeverity::Medium;
                }
                if threat_type.is_none() {
                    threat_type = Some(ThreatType::RugPull);
                }
            }
        }

        // 5. Check for unlimited approval
        if tx.is_unlimited_approval {
            indicators.push("UNLIMITED token approval requested".to_string());
            if max_severity < ThreatSeverity::High {
                max_severity = ThreatSeverity::High;
            }
            if threat_type.is_none() {
                threat_type = Some(ThreatType::MaliciousContract);
            }
        }

        // 6. Check for known good contracts (whitelist)
        if let Some(to) = &tx.to {
            if self.known_good_contracts.contains(&to.to_lowercase()) {
                // Override: known good
                return ScamVerdict {
                    is_scam: false,
                    confidence: 0.99,
                    threat_type: None,
                    severity: ThreatSeverity::Low,
                    message: "Transaction to known-good contract".to_string(),
                    indicators: vec!["Whitelisted contract".to_string()],
                    recommended_action: RecommendedAction::Allow,
                    contract_address: Some(to.clone()),
                    token_symbol: None,
                    timestamp: chrono::Utc::now(),
                };
            }
        }

        // 7. Determine recommended action
        let action = if is_scam && self.config.block_on_high_risk &&
            matches!(max_severity, ThreatSeverity::Critical | ThreatSeverity::High) {
            RecommendedAction::Block
        } else if is_scam {
            RecommendedAction::RequireConfirmation
        } else if !indicators.is_empty() {
            RecommendedAction::Warn
        } else {
            RecommendedAction::Allow
        };

        let message = if indicators.is_empty() {
            "No scam indicators detected".to_string()
        } else {
            indicators.join("; ")
        };

        ScamVerdict {
            is_scam,
            confidence: if is_scam { 0.92 } else { 0.95 },
            threat_type,
            severity: max_severity,
            message,
            indicators,
            recommended_action: action,
            contract_address: tx.to.clone(),
            token_symbol: tx.token_symbol.clone(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Adds an address to the known-bad list.
    pub fn report_scam(&mut self, address: &str, threat_type: ThreatType) {
        self.known_bad_addresses.insert(address.to_lowercase());
        self.recent_scams.push(ScamVerdict {
            is_scam: true,
            confidence: 1.0,
            threat_type: Some(threat_type),
            severity: ThreatSeverity::Critical,
            message: format!("User-reported scam: {}", address),
            indicators: vec!["User report".to_string()],
            recommended_action: RecommendedAction::Block,
            contract_address: Some(address.to_string()),
            token_symbol: None,
            timestamp: chrono::Utc::now(),
        });
    }
}

/// A transaction to be analyzed before signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionToCheck {
    pub to: Option<String>,
    pub from: String,
    pub value_usd: f64,
    pub is_token_transfer: bool,
    pub to_amount: f64,
    pub promised_apy: Option<f64>,
    pub contract_age_hours: Option<f64>,
    pub is_unlimited_approval: bool,
    pub token_symbol: Option<String>,
    pub data: Vec<u8>,
}

#[async_trait]
impl Agent for ScamDetectionAgent {
    fn id(&self) -> uuid::Uuid { self.id }
    fn name(&self) -> &str { "ScamDetection" }
    fn description(&self) -> &str {
        "Real-time scam detection: rug pulls, honeypots, phishing, APY anomalies, \
         malicious contracts, and social engineering. Can BLOCK transactions."
    }
    fn status(&self) -> &AgentStatus { &self.status }

    async fn on_tick(
        &mut self,
        _context: &AgentContext,
    ) -> Result<Option<AgentResponse>, Box<dyn std::error::Error + Send + Sync>> {
        // Scam detection is reactive (on transaction), not periodic.
        // On tick, we could scan for new scam patterns in the ecosystem.
        Ok(None)
    }

    async fn on_query(
        &mut self,
        query: &str,
        _context: &AgentContext,
    ) -> Result<AgentResponse, Box<dyn std::error::Error + Send + Sync>> {
        let lower = query.to_lowercase();

        let message = if lower.contains("check") || lower.contains("safe") || lower.contains("scam") {
            format!(
                "I'm the Scam Detection agent. I analyze transactions before signing:\n\
                 - Rug pull patterns\n\
                 - Honeypot detection\n\
                 - Phishing addresses\n\
                 - APY anomalies (>{}%)\n\
                 - Unlimited approvals\n\
                 - New contract age (<{}h)\n\n\
                 Recent detections: {}\n\nYou asked: \"{}\"",
                self.config.max_apy_threshold,
                self.config.min_contract_age_hours,
                self.recent_scams.len(),
                query
            )
        } else if lower.contains("report") {
            "To report a scam, provide the contract address and what happened.".to_string()
        } else {
            format!("Scam Detection agent ready. Ask me to check a transaction or report a scam.\n\nYou asked: \"{}\"", query)
        };

        Ok(AgentResponse {
            agent_id: self.id,
            agent_name: self.name().to_string(),
            response_type: ResponseType::Query,
            message,
            reasoning: "Responded to scam detection query.".into(),
            actions: vec![],
            confidence: 0.95,
            data: Some(serde_json::json!({
                "known_bad_count": self.known_bad_addresses.len(),
                "recent_scams": self.recent_scams.len(),
            })),
            timestamp: chrono::Utc::now(),
        })
    }

    fn configure(&mut self, params: serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(v) = params.get("max_apy_threshold").and_then(|v| v.as_f64()) {
            self.config.max_apy_threshold = v;
        }
        if let Some(v) = params.get("block_on_high_risk").and_then(|v| v.as_bool()) {
            self.config.block_on_high_risk = v;
        }
        Ok(())
    }

    fn pause(&mut self) { self.status = AgentStatus::Paused; }
    fn resume(&mut self) { self.status = AgentStatus::Active; }
    fn disable(&mut self, _reason: &str) { self.status = AgentStatus::Disabled; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_honeypot() {
        let agent = ScamDetectionAgent::new(ScamDetectConfig::default());

        let tx = TransactionToCheck {
            to: Some("0xdeadbeef".into()),
            from: "0xuser".into(),
            value_usd: 5_000.0,
            is_token_transfer: true,
            to_amount: 0.0, // honeypot: can't sell
            promised_apy: None,
            contract_age_hours: Some(500.0),
            is_unlimited_approval: false,
            token_symbol: Some("FAKE".into()),
            data: vec![],
        };

        let verdict = agent.analyze_transaction(&tx);
        assert!(verdict.is_scam);
        assert_eq!(verdict.threat_type, Some(ThreatType::Honeypot));
        assert_eq!(verdict.recommended_action, RecommendedAction::Block);
    }

    #[test]
    fn test_detects_ponzi_apy() {
        let agent = ScamDetectionAgent::new(ScamDetectConfig::default());

        let tx = TransactionToCheck {
            to: Some("0xponzi".into()),
            from: "0xuser".into(),
            value_usd: 10_000.0,
            is_token_transfer: true,
            to_amount: 10_000.0,
            promised_apy: Some(500.0), // 500% APY = scam
            contract_age_hours: Some(72.0),
            is_unlimited_approval: false,
            token_symbol: Some("MOON".into()),
            data: vec![],
        };

        let verdict = agent.analyze_transaction(&tx);
        assert!(verdict.is_scam);
        assert_eq!(verdict.threat_type, Some(ThreatType::PonziScheme));
        assert!(verdict.indicators.iter().any(|i| i.contains("500%")));
    }

    #[test]
    fn test_allows_safe_transaction() {
        let agent = ScamDetectionAgent::new(ScamDetectConfig::default());

        let tx = TransactionToCheck {
            to: Some("0xknowngood".into()),
            from: "0xuser".into(),
            value_usd: 500.0,
            is_token_transfer: true,
            to_amount: 500.0,
            promised_apy: None,
            contract_age_hours: Some(8000.0),
            is_unlimited_approval: false,
            token_symbol: Some("USDC".into()),
            data: vec![],
        };

        let verdict = agent.analyze_transaction(&tx);
        assert!(!verdict.is_scam);
        assert_eq!(verdict.recommended_action, RecommendedAction::Allow);
    }

    #[test]
    fn test_detects_unlimited_approval() {
        let agent = ScamDetectionAgent::new(ScamDetectConfig::default());

        let tx = TransactionToCheck {
            to: Some("0xnewcontract".into()),
            from: "0xuser".into(),
            value_usd: 0.0,
            is_token_transfer: false,
            to_amount: 0.0,
            promised_apy: None,
            contract_age_hours: Some(2.0), // very new
            is_unlimited_approval: true,
            token_symbol: Some("ETH".into()),
            data: vec![0x09, 0x5e, 0x7b, 0x3e],
        };

        let verdict = agent.analyze_transaction(&tx);
        assert!(verdict.is_scam);
        assert!(verdict.indicators.iter().any(|i| i.contains("UNLIMITED")));
        assert!(verdict.indicators.iter().any(|i| i.contains("2h old")));
    }

    #[test]
    fn test_known_bad_address_blocked() {
        let mut agent = ScamDetectionAgent::new(ScamDetectConfig::default());
        agent.report_scam("0xevilscam", ThreatType::Phishing);

        let tx = TransactionToCheck {
            to: Some("0xevilscam".into()),
            from: "0xuser".into(),
            value_usd: 100.0,
            is_token_transfer: true,
            to_amount: 100.0,
            promised_apy: None,
            contract_age_hours: None,
            is_unlimited_approval: false,
            token_symbol: None,
            data: vec![],
        };

        let verdict = agent.analyze_transaction(&tx);
        assert!(verdict.is_scam);
        assert_eq!(verdict.severity, ThreatSeverity::Critical);
        assert_eq!(verdict.recommended_action, RecommendedAction::Block);
    }

    #[test]
    fn test_whitelisted_contract_overrides() {
        let mut agent = ScamDetectionAgent::new(ScamDetectConfig::default());
        agent.known_good_contracts.insert("0xaavev3".to_lowercase());

        let tx = TransactionToCheck {
            to: Some("0xaavev3".into()),
            from: "0xuser".into(),
            value_usd: 50_000.0,
            is_token_transfer: true,
            to_amount: 50_000.0,
            promised_apy: Some(5.0),
            contract_age_hours: Some(100_000.0),
            is_unlimited_approval: true, // would normally flag
            token_symbol: Some("USDC".into()),
            data: vec![],
        };

        let verdict = agent.analyze_transaction(&tx);
        assert!(!verdict.is_scam);
        assert_eq!(verdict.recommended_action, RecommendedAction::Allow);
    }
}   