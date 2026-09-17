//! ODAMP DEX Aggregator
//!
//! Aggregates liquidity across DEX protocols (Uniswap V3, Curve,
//! Jupiter, 1inch, Balancer, etc.) to find the best execution path.
//!
//! For EVM chains: uses 1inch API + direct Uniswap/Curve queries
//! For Solana: uses Jupiter API
//! For multi-hop: finds optimal route through intermediate tokens

pub mod aggregator;
pub mod routes;
pub mod slippage;
pub mod error;

pub use aggregator::DexAggregator;
pub use routes::{DexRoute, RouteStep};
pub use error::DexAggError;   