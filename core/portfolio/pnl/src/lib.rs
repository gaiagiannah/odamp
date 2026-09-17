//! P&L calculation engine: real-time, historical, and risk-adjusted.

pub mod realtime;
pub mod historical;
pub mod risk_adjusted;
pub mod error;

pub use realtime::RealtimePnl;
pub use risk_adjusted::RiskAdjustedPnl;
pub use error::PnlError;   