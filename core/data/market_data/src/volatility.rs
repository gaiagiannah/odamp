//! Volatility calculations: realized volatility, historical volatility,
//! annualized metrics, and risk-adjusted return measures.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::error::MarketDataError;

/// Volatility and risk metrics for an asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityMetrics {
    pub symbol: String,
    pub realized_vol_24h: f64,       // annualized
    pub realized_vol_7d: f64,        // annualized
    pub realized_vol_30d: f64,       // annualized
    pub max_drawdown_30d: f64,       // percentage
    pub sharpe_ratio_30d: f64,
    pub sortino_ratio_30d: f64,
    pub var_95_1d: f64,              // Value at Risk (1-day, 95%)
    pub cvar_95_1d: f64,             // Conditional VaR
    pub beta_to_btc: Option<f64>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Calculates volatility metrics from price history.
pub struct VolatilityCalculator {
    /// Rolling price history (timestamp, price) per symbol.
    history: std::collections::HashMap<String, VecDeque<(chrono::DateTime<chrono::Utc>, f64)>>,
    max_history: usize,
}

impl VolatilityCalculator {
    pub fn new(max_history_points: usize) -> Self {
        Self {
            history: std::collections::HashMap::new(),
            max_history: max_history_points,
        }
    }

    pub fn add_price(&mut self, symbol: &str, timestamp: chrono::DateTime<chrono::Utc>, price: f64) {
        let entry = self.history.entry(symbol.to_string()).or_default();
        entry.push_back((timestamp, price));
        if entry.len() > self.max_history {
            entry.pop_front();
        }
    }

    /// Calculates annualized realized volatility from price history.
    pub fn calculate(&self, symbol: &str) -> Result<VolatilityMetrics, MarketDataError> {
        let history = self
            .history
            .get(symbol)
            .ok_or_else(|| MarketDataError::NoData(symbol.to_string()))?;

        if history.len() < 2 {
            return Err(MarketDataError::InsufficientData {
                have: history.len(),
                need: 2,
            });
        }

        let prices: Vec<f64> = history.iter().map(|(_, p)| *p).collect();
        let returns = self.calculate_returns(&prices);

        let vol_24h = self.annualized_vol(&returns, 365 * 24); // hourly data → 24h periods
        let vol_7d = self.annualized_vol(&returns, 365);
        let vol_30d = self.annualized_vol(&returns, 365);

        let max_dd = self.max_drawdown(&prices);
        let sharpe = self.sharpe_ratio(&returns, 0.0);
        let sortino = self.sortino_ratio(&returns, 0.0);
        let var_95 = self.value_at_risk(&returns, 0.95);
        let cvar_95 = self.conditional_var(&returns, 0.95);

        Ok(VolatilityMetrics {
            symbol: symbol.to_string(),
            realized_vol_24h: vol_24h,
            realized_vol_7d: vol_7d,
            realized_vol_30d: vol_30d,
            max_drawdown_30d: max_dd,
            sharpe_ratio_30d: sharpe,
            sortino_ratio_30d: sortino,
            var_95_1d: var_95,
            cvar_95_1d: cvar_95,
            beta_to_btc: None, // requires BTC returns for correlation
            updated_at: chrono::Utc::now(),
        })
    }

    fn calculate_returns(&self, prices: &[f64]) -> Vec<f64> {
        prices
            .windows(2)
            .filter_map(|w| {
                if w[0] > 0.0 {
                    Some((w[1] - w[0]) / w[0])
                } else {
                    None
                }
            })
            .collect()
    }

    fn annualized_vol(&self, returns: &[f64], periods_per_year: f64) -> f64 {
        if returns.len() < 2 {
            return 0.0;
        }
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
        (variance.sqrt() * periods_per_year.sqrt()) * 100.0
    }

    fn max_drawdown(&self, prices: &[f64]) -> f64 {
        let mut peak = f64::MIN;
        let mut max_dd = 0.0;
        for &price in prices {
            if price > peak {
                peak = price;
            }
            if peak > 0.0 {
                let dd = (peak - price) / peak;
                if dd > max_dd {
                    max_dd = dd;
                }
            }
        }
        max_dd * 100.0
    }

    fn sharpe_ratio(&self, returns: &[f64], risk_free: f64) -> f64 {
        if returns.len() < 2 {
            return 0.0;
        }
        let mean = returns.iter().sum::<f64>() / returns.len() as f64 - risk_free;
        let std = self.std_dev(returns);
        if std == 0.0 { 0.0 } else { mean / std * (365.0_f64).sqrt() }
    }

    fn sortino_ratio(&self, returns: &[f64], risk_free: f64) -> f64 {
        let downside: Vec<f64> = returns
            .iter()
            .filter(|r| **r < risk_free)
            .map(|r| (**r - risk_free).powi(2))
            .collect();
        if downside.is_empty() {
            return 0.0;
        }
        let mean = returns.iter().sum::<f64>() / returns.len() as f64 - risk_free;
        let downside_dev = (downside.iter().sum::<f64>() / downside.len() as f64).sqrt();
        if downside_dev == 0.0 { 0.0 } else { mean / downside_dev * (365.0_f64).sqrt() }
    }

    fn value_at_risk(&self, returns: &[f64], confidence: f64) -> f64 {
        let mut sorted = returns.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = ((1.0 - confidence) * sorted.len() as f64) as usize;
        let index = index.min(sorted.len() - 1);
        -sorted[index] * 100.0
    }

    fn conditional_var(&self, returns: &[f64], confidence: f64) -> f64 {
        let mut sorted = returns.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = ((1.0 - confidence) * sorted.len() as f64) as usize;
        let index = index.min(sorted.len() - 1);
        let tail: Vec<f64> = sorted[..=index].to_vec();
        if tail.is_empty() { return 0.0; }
        -(tail.iter().sum::<f64>() / tail.len() as f64) * 100.0
    }

    fn std_dev(&self, data: &[f64]) -> f64 {
        if data.len() < 2 { return 0.0; }
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        (data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64).sqrt()
    }
}

impl Default for VolatilityCalculator {
    fn default() -> Self {
        Self::new(8760) // 1 year of hourly data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volatility_calculation() {
        let mut calc = VolatilityCalculator::new(100);

        // Add 30 days of hourly prices with some volatility
        let mut price = 100.0;
        let mut time = chrono::Utc::now() - chrono::Duration::hours(720);
        for i in 0..720 {
            price *= 1.0 + (i as f64 % 7 - 3.0) * 0.001;
            calc.add_price("TEST", time, price);
            time += chrono::Duration::hours(1);
        }

        let metrics = calc.calculate("TEST").unwrap();
        assert!(metrics.realized_vol_30d > 0.0);
        assert!(metrics.max_drawdown_30d >= 0.0);
        assert!(metrics.var_95_1d > 0.0);
    }
}   