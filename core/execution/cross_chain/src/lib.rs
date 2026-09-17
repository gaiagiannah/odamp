//! ODAMP Cross-Chain Execution Engine
//!
//! Intent-based cross-chain asset transfer. The user says "move $5K USDC
//! from Ethereum to Solana" and the platform selects the optimal route
//! considering cost, speed, security, and current congestion.
//!
//! Supported bridges:
//! - LayerZero (50+ chains)
//! - Wormhole / Portal (30+ chains)
//! - Axelar (50+ chains)
//! - Circle CCTP V2 (USDC-native, 10+ chains)
//! - Stargate Finance (10+ chains)
//! - NEAR Intents (multi-chain)
//!
//! Design principle: The user never interacts with a bridge directly.
//! The intent engine resolves the optimal path automatically.

pub mod intent;
pub mod bridge;
pub mod routes;
pub mod security;
pub mod status;
pub mod error;

pub use intent::{IntentEngine, TransferIntent};
pub use bridge::{Bridge, BridgeType, BridgeQuote};
pub use routes::CrossChainRoute;
pub use error::CrossChainError;   