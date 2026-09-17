//! Cross-chain route selection and optimization.

use serde::{Deserialize, Serialize};
use crate::bridge::{Bridge, BridgeQuote};
use crate::error::CrossChainError;

/// A complete cross-chain transfer route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainRoute {
    pub bridge_id: String,
    pub bridge_name: String,
    pub from_chain: String,
    pub to_chain: String,
    pub token: String,
    pub amount: f64,
    pub amount_received: f64,
    pub total_cost_usd: f64,
    pub estimated_time_secs: u64,
    pub security_score: f64,
    pub alternative_routes: Vec<RouteAlternative>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteAlternative {
    pub bridge_id: String,
    pub total_cost_usd: f64,
    pub estimated_time_secs: u64,
    pub security_score: f64,
}

/// Scores and selects the optimal cross-chain route.
pub struct RouteSelector {
    bridges: Vec<Bridge>,
    weight_cost: f64,
    weight_speed: f64,
    weight_security: f64,
}

impl RouteSelector {
    pub fn new(bridges: Vec<Bridge>) -> Self {
        Self {
            bridges,
            weight_cost: 0.4,
            weight_speed: 0.25,
            weight_security: 0.35,
        }
    }

    /// Selects the best bridge for a transfer.
    pub fn select(
        &self,
        from_chain: &str,
        to_chain: &str,
        token: &str,
        amount: f64,
    ) -> Result<CrossChainRoute, CrossChainError> {
        // Filter: operational, no exploit, supports both chains, within limits
        let eligible: Vec<&Bridge> = self.bridges
            .iter()
            .filter(|b| {
                b.is_operational
                    && !b.has_active_exploit
                    && b.supported_chains.contains(&from_chain.to_string())
                    && b.supported_chains.contains(&to_chain.to_string())
                    && amount <= b.max_transfer_usd
                    && amount >= b.min_transfer_usd
                    // CCTP only for USDC
                    && (b.bridge_type != crate::bridge::BridgeType::CircleCctp || token == "USDC")
            })
            .collect();

        if eligible.is_empty() {
            return Err(CrossChainError::NoRoute {
                from_chain: from_chain.to_string(),
                to_chain: to_chain.to_string(),
            });
        }

        // Score each bridge
        let mut scored: Vec<(&Bridge, f64)> = eligible
            .iter()
            .map(|b| {
                let cost_score = 1.0 - (b.base_fee_usd / 5.0).min(1.0);
                let speed_score = 1.0 - ((b.avg_time_secs as f64) / 600.0).min(1.0);
                let security_score = b.security_score / 100.0;

                let total = cost_score * self.weight_cost
                    + speed_score * self.weight_speed
                    + security_score * self.weight_security;

                (b, total)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let (best, _) = &scored[0];

        // Build route
        let fee = best.base_fee_usd + (amount * 0.001); // 0.1% bridge fee
        let gas = 2.0; // estimated gas
        let received = amount - (fee / 1000.0); // simplified

        let alternatives: Vec<RouteAlternative> = scored
            .iter()
            .skip(1)
            .take(3)
            .map(|(b, _)| RouteAlternative {
                bridge_id: b.id.clone(),
                total_cost_usd: b.base_fee_usd + (amount * 0.001),
                estimated_time_secs: b.avg_time_secs,
                security_score: b.security_score,
            })
            .collect();

        Ok(CrossChainRoute {
            bridge_id: best.id.clone(),
            bridge_name: best.name.clone(),
            from_chain: from_chain.to_string(),
            to_chain: to_chain.to_string(),
            token: token.to_string(),
            amount,
            amount_received: received,
            total_cost_usd: fee + gas,
            estimated_time_secs: best.avg_time_secs,
            security_score: best.security_score,
            alternative_routes: alternatives,
        })
    }
}

impl Default for RouteSelector {
    fn default() -> Self {
        Self::new(vec![
            Bridge::layer_zero(),
            Bridge::wormhole(),
            Bridge::axelar(),
            Bridge::circle_cctp(),
            Bridge::stargate(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_selection_usdc() {
        let selector = RouteSelector::default();
        let route = selector.select("ethereum", "solana", "USDC", 5_000.0).unwrap();

        // CCTP should win for USDC (lowest fee, highest security)
        assert_eq!(route.bridge_id, "circle-cctp");
        assert!(route.security_score > 95.0);
        assert!(route.total_cost_usd < 5.0);
    }

    #[test]
    fn test_route_selection_generic_token() {
        let selector = RouteSelector::default();
        let route = selector.select("ethereum", "sui", "PEPE", 100.0).unwrap();

        // CCTP not available for non-USDC
        assert_ne!(route.bridge_id, "circle-cctp");
    }

    #[test]
    fn test_no_route_unsupported_chains() {
        let selector = RouteSelector::default();
        let result = selector.select("ethereum", "dogecoin", "USDC", 100.0);
        assert!(result.is_err());
    }
}   