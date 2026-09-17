//! ODAMP Market Data Service
//!
//! Real-time price feeds, volatility calculations, funding rates,
//! and liquidity depth data from multiple sources.
//!
//! Primary sources:
//! - Chainlink oracles (on-chain, authoritative)
//! - Exchange APIs (Coinbase, Binance, Kraken) for real-time
//! - DEX pool data for on-chain pricing
//! - LME/COMEX for commodity prices (via Chainlink)

pub mod price_feed;
pub mod volatility;
pub mod liquidity;
pub mod funding;
pub mod commodity;
pub mod config;
pub mod error;

pub use price_feed::{PriceFeed, PriceQuote, PriceSource};
pub use volatility::VolatilityCalculator;
pub use error::MarketDataError;   