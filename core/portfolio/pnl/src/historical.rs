//! Historical P&L: time-series of portfolio value for charting.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value_usd: f64,
    pub daily_change_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPnl {
    pub points: Vec<HistoricalPoint>,
    pub total_return_pct: f64,
    pub cagr_pct: f64,
    pub best_day_pct: f64,
    pub worst_day_pct: f64,
    pub win_rate_pct: f64, // % of days with positive return
}

pub struct HistoricalTracker {
    history: VecDeque<HistoricalPoint>,
    max_points: usize,
}

impl HistoricalTracker {
    pub fn new(max_points: usize) -> Self {
        Self { history: VecDeque::with_capacity(max_points), max_points }
    }

    pub fn record(&mut self, timestamp: chrono::DateTime<chrono::Utc>, value_usd: f64) {
        let daily_change = if let Some(last) = self.history.back() {
            if last.value_usd > 0.0 {
                ((value_usd - last.value_usd) / last.value_usd) * 100.0
            } else { 0.0 }
        } else { 0.0 };

        self.history.push_back(HistoricalPoint {
            timestamp,
            value_usd,
            daily_change_pct: daily_change,
        });

        if self.history.len() > self.max_points {
            self.history.pop_front();
        }
    }

    pub fn summary(&self) -> Option<HistoricalPnl> {
        if self.history.len() < 2 {
            return None;
        }

        let first = self.history.front().unwrap();
        let last = self.history.back().unwrap();

        let total_return = ((last.value_usd - first.value_usd) / first.value_usd) * 100.0;
        let days = (last.timestamp - first.timestamp).num_days() as f64;
        let years = days / 365.0;
        let cagr = if years > 0.0 && first.value_usd > 0.0 {
            ((last.value_usd / first.value_usd).powf(1.0 / years) - 1.0) * 100.0
        } else { 0.0 };

        let positive_days = self.history.iter().filter(|p| p.daily_change_pct > 0.0).count();
        let win_rate = (positive_days as f64 / self.history.len() as f64) * 100.0;

        let best = self.history.iter().map(|p| p.daily_change_pct).fold(f64::MIN, f64::max);
        let worst = self.history.iter().map(|p| p.daily_change_pct).fold(f64::MAX, f64::min);

        Some(HistoricalPnl {
            points: self.history.iter().cloned().collect(),
            total_return_pct: total_return,
            cagr_pct: cagr,
            best_day_pct: best,
            worst_day_pct: worst,
            win_rate_pct: win_rate,
        })
    }
}

impl Default for HistoricalTracker {
    fn default() -> Self {
        Self::new(365 * 24) // 1 year of hourly data
    }
}   