//! ODAMP Portfolio State Engine
//!
//! Maintains real-time portfolio state by combining:
//! - On-chain position data (from indexer)
//! - Market prices (from market data service)
//! - Protocol-specific valuations (DeFi positions)
//!
//! Outputs: complete portfolio snapshot with USD valuations,
//! risk metrics, and allocation breakdown.

pub mod snapshot;
pub valuations;
pub mod allocation;
pub mod error;

pub use snapshot::PortfolioSnapshot;
pub use valuations::ValuationEngine;
pub use error::PortfolioError;   