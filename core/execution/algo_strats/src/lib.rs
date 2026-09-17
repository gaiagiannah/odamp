//! ODAMP Algorithmic Strategy Engine
//!
//! Pre-built and custom algorithmic trading strategies:
//! - DCA (Dollar Cost Averaging)
//! - Grid Trading
//! - Mean Reversion
//! - Momentum
//! - Arbitrage (cross-venue, cross-chain, triangular)
//! - Yield Farming Optimization
//! - Portfolio Rebalancing
//! - Custom (via QuantConnect LEAN integration)
//!
//! Each strategy is a stateful engine that:
//! 1. Monitors market conditions
//! 2. Generates signals (buy/sell/hold)
//! 3. Produces orders (routed through the OrderRouter)
//! 4. Tracks performance (P&L, drawdown, win rate)
//! 5. Respects user-defined limits (max position, max daily spend)

pub mod strategy;
pub mod dca;
pub mod grid;
pub mod mean_reversion;
pub mod momentum;
pub mod arbitrage;
pub mod yield_opt;
pub mod performance;
pub mod error;

pub use strategy::{Strategy, StrategyConfig, StrategySignal, StrategyState};
pub use dca::DcaStrategy;
pub use grid::GridStrategy;
pub use mean_reversion::MeanReversionStrategy;
pub use momentum::MomentumStrategy;
pub use arbitrage::ArbitrageStrategy;
pub use performance::StrategyPerformance;
pub use error::StrategyError;   