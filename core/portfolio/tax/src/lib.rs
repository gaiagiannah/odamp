//! ODAMP Tax Engine
//!
//! Automated cost basis tracking, gain/loss calculation, wash sale
//! detection, and jurisdiction-specific tax report generation.
//!
//! Supported jurisdictions (v1):
//! - United States (IRS: FIFO, specific ID, wash sale rules, 1099-DA)
//! - European Union (DAC8, country-specific rules)
//! - United Kingdom (HMRC)
//!
//! DeFi-specific event classification:
//! - Staking rewards (ordinary income)
//! - LP fees (ordinary income)
//! - Airdrops (ordinary income at receipt)
//! - Governance tokens (ordinary income)
//! - Yield (interest)
//! - NFT sales (capital gain)
//! - Protocol airdrops (capital gain or income depending on jurisdiction)

pub mod cost_basis;
pub mod gain_loss;
pub mod wash_sale;
pub mod jurisdiction;
pub mod report;
pub mod defi_events;
pub mod error;

pub use cost_basis::CostBasisTracker;
pub use gain_loss::GainLossCalculator;
pub use wash_sale::WashSaleDetector;
pub use jurisdiction::Jurisdiction;
pub use error::TaxError;   