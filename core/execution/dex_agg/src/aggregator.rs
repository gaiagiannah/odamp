//! DEX aggregator: queries multiple DEX APIs and returns the best route.

use serde::{Deserialize, Serialize};
use crate::routes::DexRoute;
use crate::slippage::{SlippageConfig, validate_slippage, estimate_slippage};
use crate::error::DexAggError;

/// A swap request to the aggregator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRequest {
    pub chain: String,
    pub from_token: String,
    pub to_token: String,
    pub amount: f64,
    pub amount_usd: f64,
    pub slippage: SlippageConfig,
    pub recipient: String,
    pub deadline_secs: u64,
}

/// The aggregator queries multiple DEX sources and returns the best route.
pub struct DexAggregator {
    slippage_config: SlippageConfig,
    max_routes_to_check: usize,
}

impl DexAggregator {
    pub fn new() -> Self {
        Self {
            slippage_config: SlippageConfig::default(),
            max_routes_to_check: 5,
        }
    }

    /// Finds the best route for a swap.
    pub async fn find_best_route(&self, request: &SwapRequest) -> Result<DexRoute, DexAggError> {
        tracing::info!(
            chain = %request.chain,
            from = %request.from_token,
            to = %request.to_token,
            amount = request.amount,
            "Finding best DEX route"
        );

        // Production: query 1inch API, Jupiter API, direct Uniswap/Curve
        // For development: simulate route selection

        let candidate_routes = self.fetch_routes(request).await?;

        if candidate_routes.is_empty() {
            return Err(DexAggError::NoRoute {
                from: request.from_token.clone(),
                to: request.to_token.clone(),
                chain: request.chain.clone(),
            });
        }

        // Sort by total cost (fee + slippage)
        let mut sorted = candidate_routes;
        sorted.sort_by(|a, b| {
            a.total_cost_bps().partial_cmp(&b.total_cost_bps()).unwrap()
        });

        // Validate top route
        let best = sorted[0].clone();
        let slippage_bps = best.slippage_pct() * 100.0;
        validate_slippage(slippage_bps, &request.slippage)
            .map_err(|e| DexAggError::ExecutionFailed(e))?;

        Ok(best)
    }

    async fn fetch_routes(&self, request: &SwapRequest) -> Result<Vec<DexRoute>, DexAggError> {
        // Production implementation would:
        // 1. Query 1inch: GET https://api.1inch.io/v5.0/{chain}/swap
        // 2. Query Jupiter: GET https://quote-api.jup.ag/v6/quote
        // 3. Query Uniswap subgraph for direct pool rates
        // 4. Query Curve for stablecoin pairs
        // 5. Combine and return all valid routes

        // Development: return simulated routes
        let base_price = request.amount_usd;

        let routes = vec![
            DexRoute {
                source_dex: "1inch".into(),
                chain: request.chain.clone(),
                from_token: request.from_token.clone(),
                to_token: request.to_token.clone(),
                from_amount: request.amount,
                to_amount_min: base_price * 0.995,
                to_amount_expected: base_price * 0.998,
                steps: vec![],
                total_fee_bps: 15.0,
                estimated_gas_usd: 2.5,
                execution_time_ms: 1500,
                is_optimal: false,
            },
            DexRoute {
                source_dex: "uniswap-v3".into(),
                chain: request.chain.clone(),
                from_token: request.from_token.clone(),
                to_token: request.to_token.clone(),
                from_amount: request.amount,
                to_amount_min: base_price * 0.993,
                to_amount_expected: base_price * 0.997,
                steps: vec![],
                total_fee_bps: 30.0,
                estimated_gas_usd: 3.0,
                execution_time_ms: 2000,
                is_optimal: false,
            },
            DexRoute {
                source_dex: "curve".into(),
                chain: request.chain.clone(),
                from_token: request.from_token.clone(),
                to_token: request.to_token.clone(),
                from_amount: request.amount,
                to_amount_min: base_price * 0.998,
                to_amount_expected: base_price * 0.999,
                steps: vec![],
                total_fee_bps: 4.0,
                estimated_gas_usd: 1.5,
                execution_time_ms: 2000,
                is_optimal: false,
            },
        ];

        Ok(routes.into_iter().take(self.max_routes_to_check).collect())
    }
}

impl Default for DexAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_best_route() {
        let agg = DexAggregator::new();
        let request = SwapRequest {
            chain: "ethereum".into(),
            from_token: "USDC".into(),
            to_token: "DAI".into(),
            amount: 10_000.0,
            amount_usd: 10_000.0,
            slippage: SlippageConfig::default(),
            recipient: "0x1234...".into(),
            deadline_secs: 120,
        };

        let route = agg.find_best_route(&request).await.unwrap();
        // Curve should win for stablecoin pairs (lowest fee)
        assert_eq!(route.source_dex, "curve");
        assert!(route.total_cost_bps() < 50.0);
    }
}   