//! Intent engine: the user expresses intent ("move $5K USDC from ETH to SOL"),
//! the platform resolves it to a concrete execution plan.

use serde::{Deserialize, Serialize};
use crate::bridge::Bridge;
use crate::routes::{RouteSelector, CrossChainRoute};
use crate::error::CrossChainError;

/// A user's transfer intent (natural language or structured).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferIntent {
    pub token: String,
    pub amount: f64,
    pub amount_usd: f64,
    pub from_chain: String,
    pub to_chain: String,
    pub recipient: String,
    pub max_fee_usd: Option<f64>,
    pub max_time_secs: Option<u64>,
    pub prefer_fastest: bool,
    pub prefer_cheapest: bool,
    pub prefer_safest: bool,
}

/// The resolved execution plan for an intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub intent_id: uuid::Uuid,
    pub route: CrossChainRoute,
    pub source_transaction: SourceTx,
    pub estimated_completion: chrono::DateTime<chrono::Utc>,
    pub status: TransferStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceTx {
    pub chain: String,
    pub from_address: String,
    pub to_address: String, // bridge contract
    pub token: String,
    pub amount: f64,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub max_fee: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    SourceConfirmed,
    Bridging,
    DestinationPending,
    Complete,
    Failed,
    Stuck,
}

/// Resolves user intents to execution plans.
pub struct IntentEngine {
    route_selector: RouteSelector,
}

impl IntentEngine {
    pub fn new(bridges: Vec<Bridge>) -> Self {
        Self {
            route_selector: RouteSelector::new(bridges),
        }
    }

    /// Resolves an intent into an execution plan.
    pub fn resolve(&self, intent: &TransferIntent) -> Result<ExecutionPlan, CrossChainError> {
        tracing::info!(
            token = %intent.token,
            amount = intent.amount,
            from = %intent.from_chain,
            to = %intent.to_chain,
            "Resolving transfer intent"
        );

        // Select optimal route
        let route = self.route_selector.select(
            &intent.from_chain,
            &intent.to_chain,
            &intent.token,
            intent.amount,
        )?;

        // Check constraints
        if let Some(max_fee) = intent.max_fee_usd {
            if route.total_cost_usd > max_fee {
                return Err(CrossChainError::GasFeeExceeded {
                    actual: route.total_cost_usd,
                    max: max_fee,
                });
            }
        }

        if let Some(max_time) = intent.max_time_secs {
            if route.estimated_time_secs > max_time {
                return Err(CrossChainError::TransferStuck {
                    timeout_secs: max_time,
                });
            }
        }

        let now = chrono::Utc::now();
        let completion = now + chrono::Duration::seconds(route.estimated_time_secs as i64);

        Ok(ExecutionPlan {
            intent_id: uuid::Uuid::new_v4(),
            route,
            source_transaction: SourceTx {
                chain: intent.from_chain.clone(),
                from_address: intent.recipient.clone(), // simplified
                to_address: "bridge_contract".into(),
                token: intent.token.clone(),
                amount: intent.amount,
                data: vec![],
                gas_limit: 500_000,
                max_fee: route.total_cost_usd,
            },
            estimated_completion: completion,
            status: TransferStatus::Pending,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_resolution() {
        let engine = IntentEngine::new(vec![
            Bridge::layer_zero(),
            Bridge::circle_cctp(),
            Bridge::wormhole(),
        ]);

        let intent = TransferIntent {
            token: "USDC".into(),
            amount: 5_000.0,
            amount_usd: 5_000.0,
            from_chain: "ethereum".into(),
            to_chain: "solana".into(),
            recipient: "0x1234...".into(),
            max_fee_usd: Some(10.0),
            max_time_secs: Some(600),
            prefer_fastest: false,
            prefer_cheapest: true,
            prefer_safest: false,
        };

        let plan = engine.resolve(&intent).unwrap();
        assert_eq!(plan.status, TransferStatus::Pending);
        assert!(plan.route.estimated_time_secs <= 600);
        assert!(plan.route.total_cost_usd <= 10.0);
    }
}   