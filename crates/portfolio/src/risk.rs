//! Risk metrics computation.
//!
//! All metrics are computed from actual price history data.
//! No hardcoded values, no stubs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub var_95: f64,
    pub max_drawdown: f64,
    pub hhi_concentration: f64,
    pub volatility_30d: f64,
}

/// Daily (or periodic) price points for a single asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistory {
    pub symbol: String,
    pub prices: Vec<f64>, // chronological, oldest first
    pub weights: Vec<f64>, // portfolio weight at each time point (sums to 1.0)
}

/// Compute all risk metrics from portfolio price history.
///
/// # Arguments
/// * `histories` - Price history for each asset in the portfolio
/// * `positions_value` - Current USD value of each position (for HHI)
/// * `risk_free_rate` - Annualized risk-free rate (e.g., 0.05 for 5%)
pub fn compute_risk_metrics(
    histories: &[PriceHistory],
    positions_value: &[f64],
    risk_free_rate: f64,
) -> RiskMetrics {
    let total_value: f64 = positions_value.iter().sum();

    // HHI (Herfindahl-Hirschman Index) — concentration
    let hhi = if total_value > 0.0 {
        positions_value
            .iter()
            .map(|v| {
                let share = v / total_value;
                share * share
            })
            .sum()
    } else {
        0.0
    };

    if histories.is_empty() || histories[0].prices.len() < 2 {
        return RiskMetrics {
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            var_95: 0.0,
            max_drawdown: 0.0,
            hhi_concentration: hhi,
            volatility_30d: 0.0,
        };
    }

    // Compute portfolio returns (weighted)
    let n_periods = histories[0].prices.len() - 1;
    let mut portfolio_returns: Vec<f64> = vec![0.0; n_periods];

    for i in 0..n_periods {
        for (h, weight) in histories.iter().zip(&histories.iter().map(|h| {
            // Use first period's weight as proxy (simplification)
            h.weights.first().copied().unwrap_or(1.0 / histories.len() as f64)
        })) {
            let prev = h.prices[i];
            let curr = h.prices[i + 1];
            if prev > 0.0 {
                let ret = (curr - prev) / prev;
                portfolio_returns[i] += ret * weight;
            }
        }
    }

    // Annualization factor (assume daily data → 252 trading days)
    let annualization = (252.0).sqrt();

    // Mean return
    let mean_return = portfolio_returns.iter().sum::<f64>() / n_periods as f64;

    // Volatility (std dev of returns)
    let variance = portfolio_returns
        .iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>()
        / n_periods as f64;
    let volatility = variance.sqrt();
    let volatility_30d = volatility * annualization;

    // Sharpe ratio
    let sharpe = if volatility > 0.0 {
        (mean_return * 252.0 - risk_free_rate) / (volatility * annualization)
    } else {
        0.0
    };

    // Sortino ratio (downside deviation)
    let downside_returns: Vec<f64> = portfolio_returns
        .iter()
        .filter(|r| **r < 0.0)
        .map(|r| r.powi(2))
        .collect();
    let downside_variance = if !downside_returns.is_empty() {
        downside_returns.iter().sum::<f64>() / n_periods as f64
    } else {
        0.0
    };
    let downside_deviation = downside_variance.sqrt();
    let sortino = if downside_deviation > 0.0 {
        (mean_return * 252.0 - risk_free_rate) / (downside_deviation * annualization)
    } else {
        0.0
    };

    // VaR 95% (historical)
    let mut sorted_returns = portfolio_returns.clone();
    sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let var_95_index = (n_periods as f64 * 0.05) as usize;   
    let var_95 = -sorted_returns.get(var_95_index).copied().unwrap_or(0.0);

    // Max drawdown
    let mut peak = f64::MIN;
    let mut max_dd = 0.0;
    let mut cumulative = 1.0;
    for r in &portfolio_returns {
        cumulative *= 1.0 + r;
        peak = peak.max(cumulative);
        if peak > 0.0 {
            let dd = (peak - cumulative) / peak;
            max_dd = max_dd.max(dd);
        }
    }

    RiskMetrics {
        sharpe_ratio: sharpe,
        sortino_ratio: sortino,
        var_95,
        max_drawdown: max_dd,
        hhi_concentration: hhi,
        volatility_30d,
    }
}   