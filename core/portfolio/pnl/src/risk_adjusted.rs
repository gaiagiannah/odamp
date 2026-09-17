//! Risk-adjusted return metrics: Sharpe, Sortino, Calmar, Omega.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAdjustedPnl {
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub omega_ratio: f64,
    pub max_drawdown_pct: f64,
    pub volatility_annualized_pct: f64,
    pub downside_volatility_pct: f64,
    pub tail_risk_pct: f64, // 5th percentile daily return
}

pub fn calculate(
    daily_returns: &[f64],
    max_drawdown_pct: f64,
) -> RiskAdjustedPnl {
    if daily_returns.len() < 2 {
        return RiskAdjustedPnl {
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            calmar_ratio: 0.0,
            omega_ratio: 0.0,
            max_drawdown_pct,
            volatility_annualized_pct: 0.0,
            downside_volatility_pct: 0.0,
            tail_risk_pct: 0.0,
        };
    }

    let mean = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
    let risk_free_daily = 0.0001; // ~4% annualized / 365

    let variance = daily_returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>() / (daily_returns.len() - 1) as f64;
    let std_dev = variance.sqrt();

    let sharpe = if std_dev > 0.0 {
        (mean - risk_free_daily) / std_dev * (365.0_f64).sqrt()
    } else { 0.0 };

    let downside: Vec<f64> = daily_returns
        .iter()
        .filter(|r| **r < risk_free_daily)
        .map(|r| (**r - risk_free_daily).powi(2))
        .collect();
    let downside_std = if !downside.is_empty() {
        (downside.iter().sum::<f64>() / downside.len() as f64).sqrt()
    } else { 0.0 };

    let sortino = if downside_std > 0.0 {
        (mean - risk_free_daily) / downside_std * (365.0_f64).sqrt()
    } else { 0.0 };

    let annual_return = mean * 365.0;
    let calmar = if max_drawdown_pct > 0.0 {
        annual_return / (max_drawdown_pct / 100.0)
    } else { 0.0 };

    let gains: Vec<f64> = daily_returns.iter().filter(|r| **r > 0.0).copied().collect();
    let losses: Vec<f64> = daily_returns.iter().filter(|r| **r < 0.0).map(|r| -r).collect();
    let gain_sum: f64 = gains.iter().sum();
    let loss_sum: f64 = losses.iter().sum();
    let omega = if loss_sum > 0.0 { gain_sum / loss_sum } else { 0.0 };

    let mut sorted = daily_returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let tail_idx = ((0.05 * sorted.len()) as usize).max(0).min(sorted.len() - 1);
    let tail_risk = sorted[tail_idx] * 100.0;

    RiskAdjustedPnl {
        sharpe_ratio: sharpe,
        sortino_ratio: sortino,
        calmar_ratio: calmar,
        omega_ratio: omega,
        max_drawdown_pct,
        volatility_annualized_pct: std_dev * (365.0_f64).sqrt() * 100.0,
        downside_volatility_pct: downside_std * (365.0_f64).sqrt() * 100.0,
        tail_risk_pct: tail_risk,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_adjusted_metrics() {
        // Simulate 30 days of returns
        let returns: Vec<f64> = vec![
            0.01, -0.005, 0.02, -0.01, 0.005,
            0.015, -0.003, 0.008, -0.02, 0.01,
            0.005, -0.001, 0.012, -0.008, 0.003,
            0.02, -0.005, 0.007, -0.015, 0.01,
            0.004, -0.002, 0.018, -0.01, 0.006,
            0.01, -0.004, 0.009, -0.007, 0.012,
        ];

        let metrics = calculate(&returns, 5.0);
        assert!(metrics.sharpe_ratio > 0.0);
        assert!(metrics.sortino_ratio > metrics.sharpe_ratio); // sortino >= sharpe when downside < total vol
        assert!(metrics.omega_ratio > 0.0);
        assert!(metrics.volatility_annualized_pct > 0.0);
        assert!(metrics.tail_risk_pct < 0.0); // worst 5% should be negative
    }

    #[test]
    fn test_insufficient_data() {
        let metrics = calculate(&[0.01], 0.0);
        assert_eq!(metrics.sharpe_ratio, 0.0);
    }
}   