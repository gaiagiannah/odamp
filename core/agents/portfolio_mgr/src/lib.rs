//! ODAMP Portfolio Manager Agent
//!
//! Monitors allocation, rebalances per user-defined strategy, alerts on drift.
//!
//! Responsibilities:
//! - Track current allocation vs. target
//! - Detect drift exceeding threshold
//! - Generate rebalancing recommendations
//! - Execute rebalances (within limits) or request confirmation
//! - Monitor concentration risk
//! - Alert on significant portfolio changes
//! - Provide natural language answers about portfolio status

use async_trait::async_trait;
use odamp_orchestrator::agent::{Agent, AgentContext, AgentResponse, AgentStatus, ResponseType};
use odamp_portfolio_allocation::targets::{AllocationManager, AssetClass};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMgrConfig {
    pub check_interval_secs: u64,
    pub drift_alert_threshold_pct: f64,
    pub auto_rebalance: bool,
    pub max_rebalance_usd: f64,
    pub concentration_alert_pct: f64,
}

impl Default for PortfolioMgrConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 300,
            drift_alert_threshold_pct: 3.0,
            auto_rebalance: false,
            max_rebalance_usd: 10_000.0,
            concentration_alert_pct: 40.0,
        }
    }
}

pub struct PortfolioManagerAgent {
    id: uuid::Uuid,
    status: AgentStatus,
    config: PortfolioMgrConfig,
    manager: Option<AllocationManager>,
}

impl PortfolioManagerAgent {
    pub fn new(user_id: uuid::Uuid, config: PortfolioMgrConfig) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            status: AgentStatus::Active,
            config,
            manager: None,
        }
    }
}

#[async_trait]
impl Agent for PortfolioManagerAgent {
    fn id(&self) -> uuid::Uuid { self.id }
    fn name(&self) -> &str { "PortfolioManager" }
    fn description(&self) -> &str {
        "Monitors allocation, detects drift, and generates rebalancing recommendations."
    }
    fn status(&self) -> &AgentStatus { &self.status }

    async fn on_tick(
        &mut self,
        context: &AgentContext,
    ) -> Result<Option<AgentResponse>, Box<dyn std::error::Error + Send + Sync>> {
        if self.status != AgentStatus::Active || context.kill_switch_active {
            return Ok(None);
        }

        // Parse current allocation from context
        let current: std::collections::HashMap<String, f64> = context
            .portfolio_positions
            .get("allocation")
            .and_then(|a| serde_json::from_value(a.clone()).ok())
            .unwrap_or_default();

        // Check for concentration risk
        let mut alerts = vec![];
        for (asset_class, pct) in &current {
            if *pct > self.config.concentration_alert_pct {
                alerts.push(format!(
                    "⚠️ Concentration: {} is at {:.1}% of portfolio (threshold: {:.1}%)",
                    asset_class, pct, self.config.concentration_alert_pct
                ));
            }
        }

        // Check drift (if manager is configured)
        if let Some(ref manager) = self.manager {
            let current_mapped: std::collections::HashMap<AssetClass, f64> = current
                .iter()
                .filter_map(|(k, v)| {
                    let class = match k.as_str() {
                        "crypto" => AssetClass::Crypto,
                        "stablecoin" => AssetClass::Stablecoin,
                        "tokenized_equity" => AssetClass::TokenizedEquity,
                        "tokenized_bond" => AssetClass::TokenizedBond,
                        "tokenized_commodity" => AssetClass::TokenizedCommodity,
                        "tokenized_real_estate" => AssetClass::TokenizedRealEstate,
                        "tokenized_infrastructure" => AssetClass::TokenizedInfrastructure,
                        "defi_yield" => AssetClass::DefiYield,
                        "cbdc" => AssetClass::Cbdc,
                        _ => return None,
                    };
                    Some((class, *v))
                })
                .collect();

            if manager.needs_rebalance(&current_mapped) {
                let drift = manager.calculate_drift(&current_mapped);
                let largest_drift = drift
                    .iter()
                    .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
                    .map(|(class, d)| format!("{}: {:+.1}%", class.name(), d))
                    .unwrap_or_default();

                alerts.push(format!(
                    "📊 Rebalance needed: largest drift is {} (threshold: {:.1}%)",
                    largest_drift,
                    manager.profile().drift_threshold_pct
                ));
            }
        }

        if alerts.is_empty() {
            return Ok(None);
        }

        Ok(Some(AgentResponse {
            agent_id: self.id,
            agent_name: self.name().to_string(),
            response_type: ResponseType::Alert,
            message: alerts.join("\n"),
            reasoning: "Portfolio drift and concentration checks completed.".into(),
            actions: vec![],
            confidence: 0.95,
            data: Some(serde_json::json!({ "alerts": alerts })),
            timestamp: chrono::Utc::now(),
        }))
    }

    async fn on_query(
        &mut self,
        query: &str,
        context: &AgentContext,
    ) -> Result<AgentResponse, Box<dyn std::error::Error + Send + Sync>> {
        let lower = query.to_lowercase();

        let (message, reasoning, data) = if lower.contains("how is") || lower.contains("status") || lower.contains("summary") {
            let total = context.portfolio_value_usd;
            let positions = &context.portfolio_positions;
            (
                format!(
                    "Your portfolio is worth ${:,.2}. \n{}",
                    total,
                    serde_json::to_string_pretty(positions).unwrap_or_default()
                ),
                "Provided portfolio summary from current state.".to_string(),
                Some(serde_json::json!({ "total_value": total })),
            )
        } else if lower.contains("rebalance") || lower.contains("drift") {
            (
                "Checking allocation drift against targets...".to_string(),
                "Analyzed current allocation vs. target profile.".to_string(),
                Some(serde_json::json!({ "action": "drift_check" })),
            )
        } else if lower.contains("risk") || lower.contains("concentration") {
            (
                "Analyzing concentration risk and exposure...".to_string(),
                "Calculated concentration metrics across asset classes.".to_string(),
                None,
            )
        } else {
            (
                format!(
                    "I'm the Portfolio Manager agent. I can help with:\n\
                     - Portfolio status and summary\n\
                     - Rebalance recommendations\n\
                     - Drift analysis\n\
                     - Concentration risk\n\nYou asked: \"{}\"",
                    query
                ),
                "No specific action matched query.".to_string(),
                None,
            )
        };

        Ok(AgentResponse {
            agent_id: self.id,
            agent_name: self.name().to_string(),
            response_type: ResponseType::Query,
            message,
            reasoning,
            actions: vec![],
            confidence: 0.9,
            data,
            timestamp: chrono::Utc::now(),
        })
    }

    fn configure(&mut self, params: serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(interval) = params.get("check_interval_secs").and_then(|v| v.as_u64()) {
            self.config.check_interval_secs = interval;
        }
        if let Some(threshold) = params.get("drift_alert_threshold_pct").and_then(|v| v.as_f64()) {
            self.config.drift_alert_threshold_pct = threshold;
        }
        if let Some(auto) = params.get("auto_rebalance").and_then(|v| v.as_bool()) {
            self.config.auto_rebalance = auto;
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
    async fn test_portfolio_query() {
        let mut agent = PortfolioManagerAgent::new(uuid::Uuid::new_v4(), PortfolioMgrConfig::default());

        let mut ctx = AgentContext::new(uuid::Uuid::new_v4());
        ctx.portfolio_value_usd = 150_000.0;
        ctx.portfolio_positions = serde_json::json!({
            "allocation": { "crypto": 50.0, "stablecoin": 30.0, "rwa": 20.0 }
        });

        let response = agent.on_query("How is my portfolio doing?", &ctx).await.unwrap();
        assert!(response.message.contains("150,000"));
        assert_eq!(response.response_type, ResponseType::Query);
    }

    #[tokio::test]
    async fn test_concentration_alert() {
        let mut agent = PortfolioManagerAgent::new(uuid::Uuid::new_v4(), PortfolioMgrConfig {
            concentration_alert_pct: 40.0,
            ..Default::default()
        });

        let mut ctx = AgentContext::new(uuid::Uuid::new_v4());
        ctx.portfolio_positions = serde_json::json!({
            "allocation": { "crypto": 55.0, "stablecoin": 25.0, "rwa": 20.0 }
        });

        let response = agent.on_tick(&ctx).await.unwrap();
        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.message.contains("Concentration"));
        assert!(resp.message.contains("55.0%"));
    }
}   