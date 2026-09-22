//! ODAMP On-Chain Indexer
//!
//! Two real data sources, no mocking:
//! 1. EvmClient — reads balances from any EVM JSON-RPC endpoint
//! 2. PriceClient — reads USD prices from CoinGecko public API

pub mod evm;
pub mod pricing;

pub use evm::EvmClient;
pub use pricing::PriceClient;   