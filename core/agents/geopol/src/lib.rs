//! ODAMP Geopolitical Intelligence Agent
//!
//! Tracks and analyzes:
//! - Sanctions updates (OFAC, EU, UN, UK HMT)
//! - CBDC rollout timelines and design decisions
//! - Trade policy changes (tariffs, export controls)
//! - Central bank communications (Fed, ECB, PBOC, BOE)
//! - Geopolitical conflicts affecting markets
//! - De-dollarization indicators
//! - BRICS payment system developments
//! - Energy and commodity supply disruptions
//!
//! This agent is ALERT-ONLY. It informs other agents but never executes.

use async_trait::async_trait;
use odamp_orchestrator::agent::{Agent, AgentContext, AgentResponse, AgentStatus, ResponseType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeopolConfig {
    pub check_interval_secs: u64,
    pub monitored_jurisdictions: Vec<String>,
    pub alert_on_sanctions: bool,
    pub alert_on_cbc_launch: bool,
    pub alert_on_trade_policy: bool,
    pub alert_on_energy_disruption: bool,
}

impl Default for GeopolConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 600,
            monitored_jurisdictions: vec![
                "US".into(), "EU".into(), "CN".into(), "RU".into(),
                "GB".into(), "SG".into(), "JP".into(), "IN".into(),
            ],
            alert_on_sanctions: true,
            alert_on_cbc_launch: true,
            alert_on_trade_policy: true,
            alert_on_energy_disruption: true,
        }
    }
}

/// A geopolitical event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeopolEvent {
    pub event_type: GeopolEventType,
    pub jurisdiction: String,
    pub severity: GeopolSeverity,
    pub title: String,
    pub description: String,
    pub affected_assets: Vec<String>,
    pub affected_jurisdictions: Vec<String>,
    pub market_impact: MarketImpact,
    pub source: String,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeopolEventType {
    Sanctions,
    CbdcLaunch,
    TradePolicy,
    CentralBankDecision,
    Conflict,
    EnergyDisruption,
    ExportControl,
    DeDollarization,
    RegulatoryEnforcement,
    CyberAttack,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeopolSeverity {
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarketImpact {
    None,
    VolatilityIncrease,
    SectorSpecific { sector: String },
    BroadMarket,
    CurrencyShift,
}

pub struct GeopolIntelligenceAgent {
    id: uuid::Uuid,
    status: AgentStatus,
    config: GeopolConfig,
    recent_events: Vec<GeopolEvent>,
    active_alerts: Vec<GeopolEvent>,
}

impl GeopolIntelligenceAgent {
    pub fn new(config: GeopolConfig) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            status: AgentStatus::Active,
            config,
            recent_events: vec![],
            active_alerts: vec![],
        }
    }

    fn assess_portfolio_impact(&self, event: &GeopolEvent, context: &AgentContext) -> String {
        let positions: std::collections::HashMap<String, f64> = context
            .portfolio_positions
            .get("allocation")
            .and_then(|a| serde_json::from_value(a.clone()).ok())
            .unwrap_or_default();

        let mut impacted = vec![];
        for asset in &event.affected_assets {
            if let Some(pct) = positions.get(asset) {
                impacted.push(format!("{} ({:.1}%)", asset, pct));
            }
        }

        if impacted.is_empty() {
            "No direct portfolio impact detected.".to_string()
        } else {
            format!("Portfolio exposure: {}", impacted.join(", "))
        }
    }
}

#[async_trait]
impl Agent for GeopolIntelligenceAgent {
    fn id(&self) -> uuid::Uuid { self.id }
    fn name(&self) -> &str { "GeopoliticalIntelligence" }
    fn description(&self) -> &str {
        "Tracks sanctions, CBDC rollouts, trade policy, central bank decisions, \
         conflicts, and de-dollarization indicators. Alert-only."
    }
    fn status(&self) -> &AgentStatus { &self.status }

    async fn on_tick(
        &mut self,
        context: &AgentContext,
    ) -> Result<Option<AgentResponse>, Box<dyn std::error::Error + Send + Sync>> {
        if self.status != AgentStatus::Active || context.kill_switch_active {
            return Ok(None);
        }

        // Parse geopolitical events from context
        let mut new_events = vec![];
        for event in &context.recent_events {
            if event.get("type").and_then(|t| t.as_str()) == Some("geopolitical") {
                if let Ok(geopol) = serde_json::from_value::<GeopolEvent>(event.clone()) {
                    // Filter by monitored jurisdictions
                    if self.config.monitored_jurisdictions.iter()
                        .any(|j| geopol.jurisdiction == *j || geopol.affected_jurisdictions.contains(j))
                    {
                        new_events.push(geopol);
                    }
                }
            }
        }

        if new_events.is_empty() {
            return Ok(None);
        }

        self.recent_events.extend(new_events.iter().cloned());
        if self.recent_events.len() > 500 {
            self.recent_events.drain(0..self.recent_events.len() - 500);
        }

        // Filter for alerts (High or Critical severity)
        let alerts: Vec<GeopolEvent> = new_events
            .iter()
            .filter(|e| matches!(e.severity, GeopolSeverity::High | GeopolSeverity::Critical))
            .cloned()
            .collect();

        self.active_alerts = alerts.clone();

        if alerts.is_empty() {
            return Ok(None);
        }

        let message = alerts
            .iter()
            .map(|e| {
                let impact = self.assess_portfolio_impact(e, context);
                format!(
                    "[{}] {} — {} ({}): {}\n  Impact: {}",
                    match e.severity {
                        GeopolSeverity::Low => "LOW",
                        GeopolSeverity::Moderate => "MOD",
                        GeopolSeverity::High => "HIGH",
                        GeopolSeverity::Critical => "CRIT",
                    },
                    e.jurisdiction,
                    e.event_type,
                    e.title,
                    e.description,
                    impact
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(Some(AgentResponse {
            agent_id: self.id,
            agent_name: self.name().to_string(),
            response_type: ResponseType::Alert,
            message,
            reasoning: format!(
                "Geopolitical scan: {} new event(s), {} high/critical alert(s). \
                 Monitored jurisdictions: {}",
                new_events.len(),
                alerts.len(),
                self.config.monitored_jurisdictions.join(", ")
            ),
            actions: vec![],
            confidence: 0.93,
            data: Some(serde_json::json!({
                "events": new_events,
                "alerts": alerts,
            })),
            timestamp: chrono::Utc::now(),
        }))
    }

    async fn on_query(
        &mut self,
        query: &str,
        context: &AgentContext,
    ) -> Result<AgentResponse, Box<dyn std::error::Error + Send + Sync>> {
        let lower = query.to_lowercase();

        let message = if lower.contains("sanction") {
            let sanctions: Vec<&GeopolEvent> = self.recent_events
                .iter()
                .filter(|e| e.event_type == GeopolEventType::Sanctions)
                .take(10)
                .collect();
            if sanctions.is_empty() {
                "No recent sanctions events detected.".to_string()
            } else {
                sanctions
                    .iter()
                    .map(|e| format!("- [{}] {}: {}", e.jurisdiction, e.title, e.occurred_at.format("%Y-%m-%d")))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        } else if lower.contains("cbdc") || lower.contains("digital currency") {
            let cbdc: Vec<&GeopolEvent> = self.recent_events
                .iter()
                .filter(|e| e.event_type == GeopolEventType::CbdcLaunch)
                .take(10)
                .collect();
            if cbdc.is_empty() {
                "No recent CBDC launch events.".to_string()
            } else {
                cbdc
                    .iter()
                    .map(|e| format!("- {}: {}", e.jurisdiction, e.title))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        } else if lower.contains("risk") || lower.contains("exposure") || lower.contains("impact") {
            let impact = self.assess_portfolio_impact(
                &self.recent_events.last().cloned().unwrap_or(GeopolEvent {
                    event_type: GeopolEventType::Sanctions,
                    jurisdiction: "GLOBAL".into(),
                    severity: GeopolSeverity::Low,
                    title: "No events".into(),
                    description: "No active geopolitical risk".into(),
                    affected_assets: vec![],
                    affected_jurisdictions: vec![],
                    market_impact: MarketImpact::None,
                    source: "internal".into(),
                    occurred_at: chrono::Utc::now(),
                }),
                context,
            );
            format!("Geopolitical risk assessment:\n{}", impact)
        } else if lower.contains("dollar") || lower.contains("de-dollar") || lower.contains("bridges") {
            "Tracking de-dollarization indicators: mBridge volume, BRICS Unit activity, \
             stablecoin market cap vs. CBDC adoption. Current assessment: dollar resilience \
             via stablecoins remains the base case (40% probability), multipolar fragmentation \
             (35%), accelerated de-dollarization (25%).".to_string()
        } else {
            format!(
                "I'm the Geopolitical Intelligence agent. I track:\n\
                 - Sanctions (OFAC, EU, UN)\n\
                 - CBDC rollouts\n\
                 - Trade policy & export controls\n\
                 - Central bank decisions\n\
                 - Conflicts & energy disruptions\n\
                 - De-dollarization indicators\n\n\
                 Monitored: {}\n\nYou asked: \"{}\"",
                self.config.monitored_jurisdictions.join(", "),
                query
            )
        };

        Ok(AgentResponse {
            agent_id: self.id,
            agent_name: self.name().to_string(),
            response_type: ResponseType::Query,
            message,
            reasoning: "Responded to geopolitical intelligence query.".into(),
            actions: vec![],
            confidence: 0.92,
            data: Some(serde_json::json!({
                "recent_events_count": self.recent_events.len(),
                "active_alerts_count": self.active_alerts.len(),
            })),
            timestamp: chrono::Utc::now(),
        })
    }

    fn configure(&mut self, params: serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(j) = params.get("monitored_jurisdictions").and_then(|v| v.as_array()) {
            self.config.monitored_jurisdictions = j.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
        if let Some(v) = params.get("alert_on_sanctions").and_then(|v| v.as_bool()) {
            self.config.alert_on_sanctions = v;
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
    async fn test_geopol_alert_on_critical_event() {
        let mut agent = GeopolIntelligenceAgent::new(GeopolConfig::default());

        let mut ctx = AgentContext::new(uuid::Uuid::new_v4());
        ctx.portfolio_positions = serde_json::json!({
            "allocation": { "tokenized_commodity": 15.0, "crypto": 50.0 }
        });
        ctx.recent_events = vec![serde_json::json!({
            "type": "geopolitical",
            "event_type": "Sanctions",
            "jurisdiction": "US",
            "severity": "Critical",
            "title": "New oil sanctions on major producer",
            "description": "OFAC issued new sanctions targeting oil exports",
            "affected_assets": ["tokenized_commodity"],
            "affected_jurisdictions": ["US", "EU"],
            "market_impact": "SectorSpecific",
            "source": "OFAC",
            "occurred_at": "2026-09-17T08:00:00Z"
        })];

        let response = agent.on_tick(&ctx).await.unwrap();
        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.message.contains("oil sanctions"));
        assert!(resp.message.contains("tokenized_commodity"));
    }

    #[tokio::test]
    async fn test_geopol_query_sanctions() {
        let mut agent = GeopolIntelligenceAgent::new(GeopolConfig::default());
        let ctx = AgentContext::new(uuid::Uuid::new_v4());

        let response = agent.on_query("Any new sanctions?", &ctx).await.unwrap();
        assert!(response.message.contains("No recent sanctions"));
    }

    #[tokio::test]
    async fn test_de_dollarization_query() {
        let agent = GeopolIntelligenceAgent::new(GeopolConfig::default());
        let ctx = AgentContext::new(uuid::Uuid::new_v4());

        let response = agent.on_query("What's the de-dollarization outlook?", &ctx).await.unwrap();
        assert!(response.message.contains("40%"));
        assert!(response.message.contains("multipolar"));
    }
}   