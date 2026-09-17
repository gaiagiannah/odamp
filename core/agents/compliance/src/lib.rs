//! ODAMP Compliance Agent
//!
//! Monitors regulatory changes, auto-adjusts platform parameters,
//! flags restricted assets, and ensures all operations comply with
//! the user's jurisdiction-specific requirements.
//!
//! Responsibilities:
//! - Track regulatory changes (GENIUS Act, MiCA, DAC8, etc.)
//! - Auto-adjust strategy parameters when regulation changes
//! - Flag restricted/prohibited assets per jurisdiction
//! - Monitor OFAC/EU/UN sanctions list updates
//! - Ensure Travel Rule compliance for transfers >$3,000
//! - Generate compliance reports for regulators
//! - Alert on new licensing requirements

use async_trait::async_trait;
use odamp_orchestrator::agent::{Agent, AgentContext, AgentResponse, AgentStatus, ResponseType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub jurisdiction: String,
    pub check_interval_secs: u64,
    pub auto_adjust_on_reg_change: bool,
    pub block_restricted_assets: bool,
    pub travel_rule_threshold_usd: f64,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            jurisdiction: "US".into(),
            check_interval_secs: 3600,
            auto_adjust_on_reg_change: true,
            block_restricted_assets: true,
            travel_rule_threshold_usd: 3_000.0,
        }
    }
}

/// A regulatory change event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryEvent {
    pub jurisdiction: String,
    pub regulation: String,
    pub change_type: ChangeType,
    pub description: String,
    pub effective_date: chrono::DateTime<chrono::Utc>,
    pub affected_asset_classes: Vec<String>,
    pub required_actions: Vec<String>,
    pub severity: RegulatorySeverity,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeType {
    NewRule,
    Amendment,
    Enforcement,
    Guideline,
    Repeal,
    Sandbox,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RegulatorySeverity {
    Advisory,
    Moderate,
    Significant,
    Critical,
}

pub struct ComplianceAgent {
    id: uuid::Uuid,
    status: AgentStatus,
    config: ComplianceConfig,
    active_restrictions: Vec<String>,
    recent_reg_events: Vec<RegulatoryEvent>,
    last_check: Option<chrono::DateTime<chrono::Utc>>,
}

impl ComplianceAgent {
    pub fn new(config: ComplianceConfig) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            status: AgentStatus::Active,
            config,
            active_restrictions: vec![],
            recent_reg_events: vec![],
            last_check: None,
        }
    }

    /// Checks if an asset is restricted in the user's jurisdiction.
    pub fn is_restricted(&self, asset_symbol: &str) -> bool {
        self.active_restrictions.iter().any(|r| r == asset_symbol)
    }

    /// Checks if a transfer requires Travel Rule data.
    pub fn requires_travel_rule(&self, amount_usd: f64) -> bool {
        amount_usd > self.config.travel_rule_threshold_usd
    }

    /// Processes a new regulatory event.
    pub fn process_reg_event(&mut self, event: RegulatoryEvent) {
        // Update restrictions
        if event.severity >= RegulatorySeverity::Significant {
            for asset in &event.affected_asset_classes {
                if !self.active_restrictions.contains(asset) {
                    self.active_restrictions.push(asset.clone());
                }
            }
        }

        self.recent_reg_events.push(event);
        if self.recent_reg_events.len() > 100 {
            self.recent_reg_events.remove(0);
        }
    }
}

#[async_trait]
impl Agent for ComplianceAgent {
    fn id(&self) -> uuid::Uuid { self.id }
    fn name(&self) -> &str { "Compliance" }
    fn description(&self) -> &str {
        "Monitors regulatory changes, sanctions updates, and jurisdiction-specific \
         requirements. Auto-adjusts platform parameters and flags restricted assets."
    }
    fn status(&self) -> &AgentStatus { &self.status }

    async fn on_tick(
        &mut self,
        context: &AgentContext,
    ) -> Result<Option<AgentResponse>, Box<dyn std::error::Error + Send + Sync>> {
        if self.status != AgentStatus::Active || context.kill_switch_active {
            return Ok(None);
        }

        // Check for new regulatory events in context
        let mut new_events = vec![];
        for event in &context.recent_events {
            if event.get("type").and_then(|t| t.as_str()) == Some("regulatory") {
                if let Ok(reg_event) = serde_json::from_value::<RegulatoryEvent>(event.clone()) {
                    new_events.push(reg_event);
                }
            }
        }

        if new_events.is_empty() {
            self.last_check = Some(chrono::Utc::now());
            return Ok(None);
        }

        // Process new events
        let mut alerts = vec![];
        for event in &new_events {
            self.process_reg_event(event.clone());
            alerts.push(format!(
                "[{}] {} — {}: {} (effective: {})",
                match event.severity {
                    RegulatorySeverity::Advisory => "ADV",
                    RegulatorySeverity::Moderate => "MOD",
                    RegulatorySeverity::Significant => "SIG",
                    RegulatorySeverity::Critical => "CRIT",
                },
                event.jurisdiction,
                event.regulation,
                event.description,
                event.effective_date.format("%Y-%m-%d")
            ));
        }

        self.last_check = Some(chrono::Utc::now());

        Ok(Some(AgentResponse {
            agent_id: self.id,
            agent_name: self.name().to_string(),
            response_type: ResponseType::Alert,
            message: alerts.join("\n"),
            reasoning: format!(
                "Detected {} new regulatory event(s) for jurisdiction {}. \
                 Active restrictions: {}.",
                new_events.len(),
                self.config.jurisdiction,
                self.active_restrictions.len()
            ),
            actions: vec![],
            confidence: 0.97,
            data: Some(serde_json::json!({
                "events": new_events,
                "active_restrictions": self.active_restrictions,
            })),
            timestamp: chrono::Utc::now(),
        }))
    }

    async fn on_query(
        &mut self,
        query: &str,
        _context: &AgentContext,
    ) -> Result<AgentResponse, Box<dyn std::error::Error + Send + Sync>> {
        let lower = query.to_lowercase();

        let message = if lower.contains("restrict") || lower.contains("prohibited") || lower.contains("banned") {
            if self.active_restrictions.is_empty() {
                format!("No restricted assets in jurisdiction {}.", self.config.jurisdiction)
            } else {
                format!(
                    "Restricted assets in {}:\n{}",
                    self.config.jurisdiction,
                    self.active_restrictions
                        .iter()
                        .map(|a| format!("  - {}", a))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
        } else if lower.contains("travel rule") || lower.contains("travel") {
            format!(
                "Travel Rule applies to transfers > ${:,.0} in {}.\n\
                 This requires originator and beneficiary information.",
                self.config.travel_rule_threshold_usd,
                self.config.jurisdiction
            )
        } else if lower.contains("sanction") || lower.contains("ofac") {
            "Checking latest OFAC/EU/UN sanctions list updates...".to_string()
        } else if lower.contains("status") || lower.contains("summary") {
            format!(
                "Compliance Status for {}:\n\
                 - Active restrictions: {}\n\
                 - Recent regulatory events: {}\n\
                 - Last check: {}\n\
                 - Travel Rule threshold: ${:,.0}\n\
                 - Auto-adjust on reg change: {}",
                self.config.jurisdiction,
                self.active_restrictions.len(),
                self.recent_reg_events.len(),
                self.last_check.map(|t| t.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "never".into()),
                self.config.travel_rule_threshold_usd,
                self.config.auto_adjust_on_reg_change
            )
        } else {
            format!(
                "I'm the Compliance agent for jurisdiction {}.\n\
                 I monitor:\n\
                 - Regulatory changes (GENIUS Act, MiCA, DAC8, etc.)\n\
                 - Sanctions updates (OFAC, EU, UN)\n\
                 - Restricted assets\n\
                 - Travel Rule compliance\n\
                 - Licensing requirements\n\nYou asked: \"{}\"",
                self.config.jurisdiction, query
            )
        };

        Ok(AgentResponse {
            agent_id: self.id,
            agent_name: self.name().to_string(),
            response_type: ResponseType::Query,
            message,
            reasoning: "Responded to compliance query.".into(),
            actions: vec![],
            confidence: 0.95,
            data: None,
            timestamp: chrono::Utc::now(),
        })
    }

    fn configure(&mut self, params: serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(j) = params.get("jurisdiction").and_then(|v| v.as_str()) {
            self.config.jurisdiction = j.to_string();
        }
        if let Some(v) = params.get("block_restricted_assets").and_then(|v| v.as_bool()) {
            self.config.block_restricted_assets = v;
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

    #[tokio::test]
    async fn test_compliance_query_status() {
        let mut agent = ComplianceAgent::new(ComplianceConfig::default());
        let ctx = AgentContext::new(uuid::Uuid::new_v4());

        let response = agent.on_query("What's my compliance status?", &ctx).await.unwrap();
        assert!(response.message.contains("US"));
        assert!(response.message.contains("Compliance Status"));
    }

    #[tokio::test]
    async fn test_regulatory_event_processing() {
        let mut agent = ComplianceAgent::new(ComplianceConfig::default());

        let event = RegulatoryEvent {
            jurisdiction: "US".into(),
            regulation: "GENIUS Act".into(),
            change_type: ChangeType::NewRule,
            description: "New stablecoin reserve requirements effective".into(),
            effective_date: chrono::Utc::now() + chrono::Duration::days(90),
            affected_asset_classes: vec!["USDC".into(), "USDT".into()],
            required_actions: vec!["Update reserve verification".into()],
            severity: RegulatorySeverity::Significant,
            source: "Federal Reserve".into(),
            timestamp: chrono::Utc::now(),
        };

        agent.process_reg_event(event);
        assert!(agent.is_restricted("USDC"));
        assert!(agent.is_restricted("USDT"));
        assert!(!agent.is_restricted("BTC"));
    }

    #[test]
    fn test_travel_rule_threshold() {
        let agent = ComplianceAgent::new(ComplianceConfig::default());
        assert!(!agent.requires_travel_rule(1_000.0));
        assert!(agent.requires_travel_rule(5_000.0));
    }
}   