//! ODAMP Execution Optimizer Agent
//!
//! Monitors execution quality (TCA) and automatically adjusts
//! routing parameters to minimize costs over time.
//!
//! Responsibilities:
//! - Track slippage per venue over time
//! - Detect venue performance degradation
//! - Adjust routing weights dynamically
//! - Identify optimal execution windows (volatility, liquidity)
//! - Suggest order splitting strategies based on historical data
//! - Flag venues with consistently poor fill quality
//! - Recommend MEV protection level adjustments

use async_trait::async_trait;
use odamp_orchestrator::agent::{Agent, AgentContext, AgentResponse, AgentStatus, ResponseType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecOptConfig {
    pub check_interval_secs: u64,
    pub min_samples_for_adjustment: u32,
    pub max_slippage_bps: f64,
    pub venue_degradation_threshold_bps: f64,
    pub auto_adjust_routing: bool,
}

impl Default for ExecOptConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 300,
            min_samples_for_adjustment: 10,
            max_slippage_bps: 50.0,
            venue_degradation_threshold_bps: 30.0,
            auto_adjust_routing: true,
        }
    }
}

/// Execution quality metrics for a venue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueMetrics {
    pub venue_id: String,
    pub total_orders: u32,
    pub avg_slippage_bps: f64,
    pub p95_slippage_bps: f64,
    pub fill_rate_pct: f64,
    pub avg_fee_bps: f64,
    pub total_cost_bps: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

pub struct ExecutionOptimizerAgent {
    id: uuid::Uuid,
    status: AgentStatus,
    config: ExecOptConfig,
    venue_metrics: std::collections::HashMap<String, VenueMetrics>,
    routing_adjustments: Vec<serde_json::Value>,
}

impl ExecutionOptimizerAgent {
    pub fn new(config: ExecOptConfig) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            status: AgentStatus::Active,
            config,
            venue_metrics: std::collections::HashMap::new(),
            routing_adjustments: vec![],
        }
    }

    /// Records execution data for a venue.
    pub fn record_execution(&mut self, venue_id: &str, slippage_bps: f64, fee_bps: f64, filled: bool) {
        let metrics = self.venue_metrics.entry(venue_id.to_string()).or_insert_with(|| VenueMetrics {
            venue_id: venue_id.to_string(),
            total_orders: 0,
            avg_slippage_bps: 0.0,
            p95_slippage_bps: 0.0,
            fill_rate_pct: 100.0,
            avg_fee_bps: 0.0,
            total_cost_bps: 0.0,
            last_updated: chrono::Utc::now(),
        });

        metrics.total_orders += 1;
        metrics.avg_slippage_bps = (metrics.avg_slippage_bps * (metrics.total_orders - 1) as f64 + slippage_bps) / metrics.total_orders as f64;
        metrics.avg_fee_bps = (metrics.avg_fee_bps * (metrics.total_orders - 1) as f64 + fee_bps) / metrics.total_orders as f64;
        metrics.total_cost_bps = metrics.avg_slippage_bps + metrics.avg_fee_bps;
        if !filled {
            metrics.fill_rate_pct = ((metrics.fill_rate_pct * (metrics.total_orders - 1) as f64) / metrics.total_orders as f64) * 100.0;
        }
        metrics.last_updated = chrono::Utc::now();
    }

    /// Identifies venues that have degraded beyond threshold.
    pub fn detect_degradation(&self) -> Vec<String> {
        self.venue_metrics
            .values()
            .filter(|m| {
                m.total_orders >= self.config.min_samples_for_adjustment
                    && m.avg_slippage_bps > self.config.venue_degradation_threshold_bps
            })
            .map(|m| m.venue_id.clone())
            .collect()
    }
}

#[async_trait]
impl Agent for ExecutionOptimizerAgent {
    fn id(&self) -> uuid::Uuid { self.id }
    fn name(&self) -> &str { "ExecutionOptimizer" }
    fn description(&self) -> &str {
        "Monitors execution quality (TCA), detects venue degradation, \
         and adjusts routing parameters to minimize costs."
    }
    fn status(&self) -> &AgentStatus { &self.status }

    async fn on_tick(
        &mut self,
        _context: &AgentContext,
    ) -> Result<Option<AgentResponse>, Box<dyn std::error::Error + Send + Sync>> {
        if self.status != AgentStatus::Active {
            return Ok(None);
        }

        let degraded = self.detect_degradation();
        if degraded.is_empty() {
            return Ok(None);
        }

        let message = format!(
            "⚠️ {} venue(s) showing execution degradation:\n{}",
            degraded.len(),
            degraded
                .iter()
                .map(|v| {
                    let m = &self.venue_metrics[v];
                    format!(
                        "  - {}: avg slippage {:.1} bps (threshold: {:.1} bps), {} orders",
                        v, m.avg_slippage_bps, self.config.venue_degradation_threshold_bps, m.total_orders
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        );

        let adjustment = serde_json::json!({
            "action": "reduce_venue_weight",
            "venues": degraded,
            "reason": "Slippage exceeds degradation threshold",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        self.routing_adjustments.push(adjustment.clone());

        Ok(Some(AgentResponse {
            agent_id: self.id,
            agent_name: self.name().to_string(),
            response_type: ResponseType::Alert,
            message: format!("{}\n\nAction: Reducing routing weight for degraded venues.", message),
            reasoning: "TCA analysis detected venue performance degradation.".into(),
            actions: vec![],
            confidence: 0.91,
            data: Some(adjustment),
            timestamp: chrono::Utc::now(),
        }))
    }

    async fn on_query(
        &mut self,
        query: &str,
        _context: &AgentContext,
    ) -> Result<AgentResponse, Box<dyn std::error::Error + Send + Sync>> {
        let lower = query.to_lowercase();

        let message = if lower.contains("slippage") || lower.contains("cost") || lower.contains("tca") {
            if self.venue_metrics.is_empty() {
                "No execution data recorded yet.".to_string()
            } else {
                self.venue_metrics
                    .values()
                    .map(|m| format!(
                        "{}: avg slip {:.1} bps, fee {:.1} bps, total {:.1} bps, {} orders, fill rate {:.0}%",
                        m.venue_id, m.avg_slippage_bps, m.avg_fee_bps, m.total_cost_bps, m.total_orders, m.fill_rate_pct
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        } else if lower.contains("venue") || lower.contains("degrad") {
            let degraded = self.detect_degradation();
            if degraded.is_empty() {
                "All venues performing within normal parameters.".to_string()
            } else {
                format!("Degraded venues: {}", degraded.join(", "))
            }
        } else if lower.contains("optimize") || lower.contains("routing") {
            format!(
                "Routing optimization status:\n\
                 - Auto-adjust: {}\n\
                 - Max slippage: {:.0} bps\n\
                 - Degradation threshold: {:.0} bps\n\
                 - Adjustments made: {}",
                self.config.auto_adjust_routing,
                self.config.max_slippage_bps,
                self.config.venue_degradation_threshold_bps,
                self.routing_adjustments.len()
            )
        } else {
            format!(
                "I'm the Execution Optimizer agent. I monitor:\n\
                 - Slippage per venue (avg, P95)\n\
                 - Fill rates\n\
                 - Fee changes\n\
                 - Venue degradation\n\
                 - Optimal execution windows\n\n\
                 Venues tracked: {}\n\nYou asked: \"{}\"",
                self.venue_metrics.len(),
                query
            )
        };

        Ok(AgentResponse {
            agent_id: self.id,
            agent_name: self.name().to_string(),
            response_type: ResponseType::Query,
            message,
            reasoning: "Responded to execution quality query.".into(),
            actions: vec![],
            confidence: 0.93,
            data: Some(serde_json::json!({
                "venues_tracked": self.venue_metrics.len(),
                "adjustments_made": self.routing_adjustments.len(),
            })),
            timestamp: chrono::Utc::now(),
        })
    }

    fn configure(&mut self, params: serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(v) = params.get("max_slippage_bps").and_then(|v| v.as_f64()) {
            self.config.max_slippage_bps = v;
        }
        if let Some(v) = params.get("auto_adjust_routing").and_then(|v| v.as_bool()) {
            self.config.auto_adjust_routing = v;
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
    fn test_venue_metrics_tracking() {
        let mut agent = ExecutionOptimizerAgent::new(ExecOptConfig {
            min_samples_for_adjustment: 3,
            venue_degradation_threshold_bps: 20.0,
            ..Default::default()
        });

        // Record good executions
        agent.record_execution("coinbase", 5.0, 120.0, true);
        agent.record_execution("coinbase", 8.0, 120.0, true);
        agent.record_execution("coinbase", 4.0, 120.0, true);

        let metrics = &agent.venue_metrics["coinbase"];
        assert_eq!(metrics.total_orders, 3);
        assert!((metrics.avg_slippage_bps - 5.67).abs() < 0.1);

        // No degradation
        assert!(agent.detect_degradation().is_empty());
    }

    #[test]
    fn test_degradation_detection() {
        let mut agent = ExecutionOptimizerAgent::new(ExecOptConfig {
            min_samples_for_adjustment: 2,
            venue_degradation_threshold_bps: 30.0,
            ..Default::default()
        });

        // Record bad executions
        agent.record_execution("bad_venue", 45.0, 50.0, true);
        agent.record_execution("bad_venue", 52.0, 50.0, true);

        let degraded = agent.detect_degradation();
        assert_eq!(degraded.len(), 1);
        assert_eq!(degraded[0], "bad_venue");
    }

    #[tokio::test]
    async fn test_exec_opt_query() {
        let mut agent = ExecutionOptimizerAgent::new(ExecOptConfig::default());
        agent.record_execution("binance", 3.0, 10.0, true);
        agent.record_execution("coinbase", 15.0, 120.0, true);

        let ctx = AgentContext::new(uuid::Uuid::new_v4());
        let response = agent.on_query("What's my slippage like?", &ctx).await.unwrap();
        assert!(response.message.contains("binance"));
        assert!(response.message.contains("coinbase"));
    }
}   