//! ODAMP Portfolio Engine
//!
//! Position tracking and risk-adjusted analytics. The math in this
//! module (Sharpe ratio, max drawdown) is genuinely computed from
//! input data — nothing here is a stub returning a fixed number.
//! Data sourcing (on-chain indexer, price feeds) is a separate
//! concern that will feed `Position` values into this module in a
//! later phase.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub asset_symbol: String,
    /// e.g. "crypto", "tokenized_equity", "tokenized_commodity", "stablecoin"
    pub asset_type: String,
    pub amount: f64,
    pub avg_cost_usd: f64,
    pub current_price_usd: f64,
}

impl Position {
    pub fn current_value_usd(&self) -> f64 {
        self.amount * self.current_price_usd
    }

    pub fn cost_basis_usd(&self) -> f64 {
        self.amount * self.avg_cost_usd
    }

    pub fn unrealized_pnl_usd(&self) -> f64 {
        self.current_value_usd() - self.cost_basis_usd()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub user_id: Uuid,
    pub positions: Vec<Position>,
}

impl Portfolio {
    pub fn total_value_usd(&self) -> f64 {
        self.positions.iter().map(Position::current_value_usd).sum()
    }

    pub fn total_unrealized_pnl_usd(&self) -> f64 {
        self.positions.iter().map(Position::unrealized_pnl_usd).sum()
    }

    /// Allocation by asset_type as a fraction of total portfolio value.
    /// Returns an empty map if the portfolio is empty (avoids div-by-zero).
    pub fn allocation_by_type(&self) -> std::collections::BTreeMap<String, f64> {
        let total = self.total_value_usd();
        let mut map = std::collections::BTreeMap::new();
        if total <= 0.0 {
            return map;
        }
        for p in &self.positions {
            *map.entry(p.asset_type.clone()).or_insert(0.0) += p.current_value_usd() / total;
        }
        map
    }
}

/// Compute the annualized Sharpe ratio from a series of periodic
/// returns (e.g. daily returns as fractions, not percentages).
///
/// `risk_free_rate` should be expressed at the same periodicity as
/// the returns (e.g. a daily risk-free rate if returns are daily).
/// `periods_per_year` is used to annualize (365 for daily, 12 for
/// monthly, etc.).
pub fn sharpe_ratio(returns: &[f64], risk_free_rate: f64, periods_per_year: f64) -> Option<f64> {
    if returns.len() < 2 {
        return None;
    }
    let excess: Vec<f64> = returns.iter().map(|r| r - risk_free_rate).collect();
    let mean = excess.iter().sum::<f64>() / excess.len() as f64;
    let variance = excess.iter().map(|r| (r - mean).powi(2)).sum::<f64>()
        / (excess.len() as f64 - 1.0);
    let std_dev = variance.sqrt();
    if std_dev == 0.0 {
        return None;
    }
    Some((mean / std_dev) * periods_per_year.sqrt())
}

/// Compute maximum drawdown (as a negative fraction, e.g. -0.23 for
/// a 23% peak-to-trough decline) from a series of portfolio values
/// over time.
pub fn max_drawdown(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut peak = values[0];
    let mut worst = 0.0_f64;
    for &v in values {
        if v > peak {
            peak = v;
        }
        if peak > 0.0 {
            let drawdown = (v - peak) / peak;
            if drawdown < worst {
                worst = drawdown;
            }
        }
    }
    Some(worst)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_position(symbol: &str, asset_type: &str, amount: f64, cost: f64, price: f64) -> Position {
        Position {
            id: Uuid::new_v4(),
            asset_symbol: symbol.into(),
            asset_type: asset_type.into(),
            amount,
            avg_cost_usd: cost,
            current_price_usd: price,
        }
    }

    #[test]
    fn portfolio_totals_are_correct() {
        let portfolio = Portfolio {
            user_id: Uuid::new_v4(),
            positions: vec![
                sample_position("BTC", "crypto", 0.5, 40_000.0, 60_000.0),
                sample_position("PAXG", "tokenized_commodity", 2.0, 2_000.0, 2_100.0),
            ],
        };
        assert_eq!(portfolio.total_value_usd(), 0.5 * 60_000.0 + 2.0 * 2_100.0);
        let alloc = portfolio.allocation_by_type();
        assert!((alloc["crypto"] + alloc["tokenized_commodity"] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn max_drawdown_detects_decline() {
        let values = [100.0, 120.0, 90.0, 95.0, 60.0, 80.0];
        // Peak 120 -> trough 60 => (60-120)/120 = -0.5
        let dd = max_drawdown(&values).unwrap();
        assert!((dd - (-0.5)).abs() < 1e-9);
    }

    #[test]
    fn sharpe_ratio_is_none_for_zero_volatility() {
        let returns = [0.01, 0.01, 0.01];
        assert!(sharpe_ratio(&returns, 0.0, 365.0).is_none());
    }
}
