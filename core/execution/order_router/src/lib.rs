//! ODAMP Smart Order Router
//!
//! Routes orders across 100+ venues (CEXs, DEXs, OTC desks) to
//! minimize slippage and maximize fill quality. Does NOT aim to beat
//! HFT on latency — aims to ensure retail users don't suffer the
//! 2–5% slippage penalty that institutional routing avoids.
//!
//! Strategies:
//! - Venue selection (best price × liquidity × fee)
//! - Order splitting (VWAP/TWAP for large orders)
//! - DEX aggregation (1inch, Uniswap, Jupiter)
//! - CEX integration (Coinbase, Kraken, Binance, OKX)
//! - Cross-chain intent routing

pub mod router;
pub mod venue;
pub mod splitting;
pub mod scoring;
pub mod error;

pub use router::{OrderRouter, RoutingDecision};
pub use venue::{Venue, VenueType, VenueQuote};
pub use splitting::SplitStrategy;
pub use error::RouterError;   