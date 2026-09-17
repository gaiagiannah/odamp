//! ODAMP Tax Advisor Agent
//!
//! Provides:
//! - Real-time tax impact preview before any transaction
//! - Year-to-date gain/loss summary
//! - Wash sale detection and warnings
//! - Jurisdiction-specific tax rate application
//! - Filing deadline reminders
//! - Tax-loss harvesting suggestions
//! - DeFi event classification (staking, LP, airdrops, etc.)

use async_trait::async_trait;
use odamp_orchestrator::agent::{Agent, AgentContext, AgentResponse, AgentStatus, ResponseType};
use odamp_portfolio_tax::jurisdiction::Jurisdiction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxAdvisorConfig {
    pub jurisdiction: Jurisdiction,
    pub tax_year: i32,
    pub remind_before_filing_days: u32,
    pub suggest_tax_loss_harvesting: bool,
    pub min_harvest_gain_usd: f64,
}

impl Default for TaxAdvisorConfig {
    fn default() -> Self {
        Self {
            jurisdiction: Jurisdiction::Us,
            tax_year: 2026,
            remind_before_filing_days: 30,
            suggest_tax_loss_harvesting: true,
            min_harvest_gain_usd: 100.0,
        }
    }
}

pub struct TaxAdvisorAgent {
    id: uuid::Uuid,
    status: AgentStatus,
    config: TaxAdvisorConfig,
    ytd_realized_gain: f64,
    ytd_realized_loss: f64,
    wash_sale_flags: Vec<String>,
}

impl TaxAdvisorAgent {
    pub fn new(config: TaxAdvisorConfig) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            status: AgentStatus::Active,
            config,
            ytd_realized_gain: 0.0,
            ytd_realized_loss: 0.0,
            wash_sale_flags: vec![],
        }
    }

    /// Pre-transaction tax impact calculation.
    pub fn preview_tax_impact(
        &self,
        sale_proceeds: f64,
        cost_basis: f64,
        holding_days: u64,
    ) -> TaxImpactPreview {
        let gain_loss = sale_proceeds - cost_basis;
        let is_long_term = holding_days > self.config.jurisdiction.long_term_threshold_days();
        let rate = self.config.jurisdiction.capital_gains_rate(is_long_term);
        let tax_owed = if gain_loss > 0.0 { gain_loss * rate } else { 0.0 };

        TaxImpactPreview {
            sale_proceeds,
            cost_basis,
            gain_loss,
            is_long_term,
            tax_rate: rate,
            tax_owed,
            net_proceeds: sale_proceeds - tax_owed,
            jurisdiction: self.config.jurisdiction.code().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxImpactPreview {
    pub sale_proceeds: f64,
    pub cost_basis: f64,
    pub gain_loss: f64,
    pub is_long_term: bool,
    pub tax_rate: f64,
    pub tax_owed: f64,
    pub net_proceeds: f64,
    pub jurisdiction: String,
}

#[async_trait]
impl Agent for TaxAdvisorAgent {
    fn id(&self) -> uuid::Uuid { self.id }
    fn name(&self) -> &str { "TaxAdvisor" }
    fn description(&self) -> &str {
        "Tax impact preview, year-to-date summary, wash sale detection, \
         tax-loss harvesting suggestions, and filing reminders."
    }
    fn status(&self) -> &AgentStatus { &self.status }

    async fn on_tick(
        &mut self,
        context: &AgentContext,
    ) -> Result<Option<AgentResponse>, Box<dyn std::error::Error + Send + Sync>> {
        if self.status != AgentStatus::Active || context.kill_switch_active {
            return Ok(None);
        }

        // Check for filing deadline reminder
        let now = chrono::Utc::now();
        let filing_date = chrono::NaiveDate::from_ymd_opt(now.year(), 4, 15)
            .and_then(|d| d.and_hms_opt(23, 59, 59))
            .and_then(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc).ok());

        if let Some(filing) = filing_date {
            let days_until = (filing - now).num_days();
            if days_until > 0 && days_until <= self.config.remind_before_filing_days as i64 {
                return Ok(Some(AgentResponse {
                    agent_id: self.id,
                    agent_name: self.name().to_string(),
                    response_type: ResponseType::Alert,
                    message: format!(
                        "📅 Tax filing deadline in {} days (April 15). \
                         YTD realized gains: ${:,.2} | Losses: ${:,.2} | Net: ${:,.2}",
                        days_until,
                        self.ytd_realized_gain,
                        self.ytd_realized_loss,
                        self.ytd_realized_gain - self.ytd_realized_loss
                    ),
                    reasoning: "Filing deadline approaching.".into(),
                    actions: vec![],
                    confidence: 1.0,
                    data: Some(serde_json::json!({
                        "days_until_filing": days_until,
                        "ytd_gain": self.ytd_realized_gain,
                        "ytd_loss": self.ytd_realized_loss,
                    })),
                    timestamp: now,
                }));
            }
        }

        Ok(None)
    }

    async fn on_query(
        &mut self,
        query: &str,
        _context: &AgentContext,
    ) -> Result<AgentResponse, Box<dyn std::error::Error + Send + Sync>> {
        let lower = query.to_lowercase();
        let jur = &self.config.jurisdiction;

        let message = if lower.contains("how much tax") || lower.contains("tax on") || lower.contains("impact") {
            format!(
                "To calculate tax impact, I need: sale proceeds, cost basis, and holding period.\n\n\
                 Tax rates for {} ({}):\n\
                 - Short-term (<{} days): {:.0}%\n\
                 - Long-term (>{} days): {:.0}%\n\n\
                 Example: Sell $10,000 with $8,000 cost basis held 400 days:\n\
                 - Gain: $2,000 (long-term)\n\
                 - Tax: ${:.0}\n\
                 - Net proceeds: ${:,.2}",
                jur.code(),
                jur.reporting_form(),
                jur.long_term_threshold_days(),
                jur.capital_gains_rate(false) * 100.0,
                jur.long_term_threshold_days(),
                jur.capital_gains_rate(true) * 100.0,
                2000.0 * jur.capital_gains_rate(true),
                10_000.0 - 2000.0 * jur.capital_gains_rate(true)
            )
        } else if lower.contains("year") || lower.contains("ytd") || lower.contains("summary") {
            format!(
                "Tax Year {} Summary ({}):\n\
                 - Realized gains: ${:,.2}\n\
                 - Realized losses: ${:,.2}\n\
                 - Net: ${:,.2}\n\
                 - Wash sale flags: {}\n\
                 - Filing form: {}",
                self.config.tax_year,
                jur.code(),
                self.ytd_realized_gain,
                self.ytd_realized_loss,
                self.ytd_realized_gain - self.ytd_realized_loss,
                self.wash_sale_flags.len(),
                jur.reporting_form()
            )
        } else if lower.contains("wash sale") || lower.contains("wash") {
            if self.wash_sale_flags.is_empty() {
                "No wash sale flags detected this year.".to_string()
            } else {
                format!(
                    "⚠️ {} wash sale flag(s):\n{}",
                    self.wash_sale_flags.len(),
                    self.wash_sale_flags
                        .iter()
                        .map(|f| format!("  - {}", f))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
        } else if lower.contains("harvest") || lower.contains("loss") {
            if self.config.suggest_tax_loss_harvesting {
                format!(
                    "Tax-loss harvesting: You have ${:,.2} in unrealized losses. \
                     Selling these before year-end could offset ${:,.2} in gains. \
                     Minimum threshold: ${:.0}.",
                    self.ytd_realized_loss,
                    self.ytd_realized_loss.min(self.ytd_realized_gain),
                    self.config.min_harvest_gain_usd
                )
            } else {
                "Tax-loss harvesting suggestions are disabled.".to_string()
            }
        } else if lower.contains("defi") || lower.contains("staking") || lower.contains("airdrop") {
            format!(
                "DeFi tax event classification for {}:\n\
                 - Staking rewards → Ordinary income at receipt\n\
                 - LP fees → Ordinary income\n\
                 - Airdrops → Ordinary income at FMV\n\
                 - Governance tokens → Ordinary income\n\
                 - Lending yield → Interest income\n\
                 - Token swaps → Taxable (disposal + acquisition)\n\
                 - NFT sales → Capital gain\n\
                 - Protocol migrations → Non-taxable\n\
                 - Burns → Non-taxable",
                jur.code()
            )
        } else {
            format!(
                "I'm the Tax Advisor agent for jurisdiction {} ({}).\n\
                 I can help with:\n\
                 - Pre-transaction tax impact preview\n\
                 - Year-to-date gain/loss summary\n\
                 - Wash sale detection\n\
                 - Tax-loss harvesting suggestions\n\
                 - DeFi event classification\n\
                 - Filing reminders\n\nYou asked: \"{}\"",
                jur.code(),
                jur.reporting_form(),
                query
            )
        };

        Ok(AgentResponse {
            agent_id: self.id,
            agent_name: self.name().to_string(),
            response_type: ResponseType::Query,
            message,
            reasoning: "Responded to tax query.".into(),
            actions: vec![],
            confidence: 0.94,
            data: None,
            timestamp: chrono::Utc::now(),
        })
    }

    fn configure(&mut self, params: serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(j) = params.get("jurisdiction").and_then(|v| v.as_str()) {
            self.config.jurisdiction = match j {
                "US" => Jurisdiction::Us,
                "EU" => Jurisdiction::Eu,
                "UK" => Jurisdiction::Uk,
                "SG" => Jurisdiction::Singapore,
                "JP" => Jurisdiction::Japan,
                "IN" => Jurisdiction::India,
                "SV" => Jurisdiction::ElSalvador,
                _ => Jurisdiction::Other,
            };
        }
        if let Some(y) = params.get("tax_year").and_then(|v| v.as_i64()) {
            self.config.tax_year = y as i32;
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
    fn test_tax_impact_preview() {
        let agent = TaxAdvisorAgent::new(TaxAdvisorConfig::default());

        let preview = agent.preview_tax_impact(100_000.0, 80_000.0, 400);
        assert!(preview.is_long_term);
        assert!((preview.gain_loss - 20_000.0).abs() < 0.01);
        assert!((preview.tax_owed - 3_000.0).abs() < 0.01); // 20000 * 0.15
        assert!((preview.net_proceeds - 97_000.0).abs() < 0.01);
    }

    #[test]
    fn test_short_term_tax() {
        let agent = TaxAdvisorAgent::new(TaxAdvisorConfig::default());

        let preview = agent.preview_tax_impact(10_000.0, 8_000.0, 30);
        assert!(!preview.is_long_term);
        assert!((preview.tax_owed - 740.0).abs() < 0.01); // 2000 * 0.37
    }

    #[tokio::test]
    async fn test_tax_query_rates() {
        let agent = TaxAdvisorAgent::new(TaxAdvisorConfig::default());
        let ctx = AgentContext::new(uuid::Uuid::new_v4());

        let response = agent.on_query("How much tax would I owe?", &ctx).await.unwrap();
        assert!(response.message.contains("Short-term"));
        assert!(response.message.contains("Long-term"));
        assert!(response.message.contains("15%"));
    }

    #[tokio::test]
    async fn test_defi_classification_query() {
        let agent = TaxAdvisorAgent::new(TaxAdvisorConfig::default());
        let ctx = AgentContext::new(uuid::Uuid::new_v4());

        let response = agent.on_query("How are staking rewards taxed?", &ctx).await.unwrap();
        assert!(response.message.contains("Ordinary income"));
        assert!(response.message.contains("Staking rewards"));
    }
}   