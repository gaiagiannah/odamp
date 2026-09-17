//! ODAMP Multi-Chain On-Chain Indexer
//!
//! Tracks transactions, positions, and protocol interactions across
//! 50+ chains in real-time. Maintains a local state database that
//! powers the portfolio engine.
//!
//! Supported chains (v1):
//! - EVM: Ethereum, Arbitrum, Base, Optimism, Polygon, BSC, Avalanche
//! - Solana
//! - Sui
//! - (v2): Aptos, TON, Cosmos, Polkadot, Cardano

pub mod chain;
pub mod evm;
pub mod solana;
pub mod sui;
pub mod positions;
pub mod defi_parser;
pub mod event_bus;
pub mod config;
pub mod error;

pub use chain::{ChainId, ChainConfig, ChainRegistry};
pub use config::IndexerConfig;
pub use error::IndexerError;
pub use positions::PositionTracker;
pub use event_bus::{EventBus, IndexerEvent};   