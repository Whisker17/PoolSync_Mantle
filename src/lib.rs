//! PoolSync: A library for synchronizing and managing liquidity pools on Mantle network
//!
//! This library provides functionality to interact with and synchronize data from
//! multiple DeFi protocols (UniswapV3, Agni, MerchantMoe) on the Mantle blockchain network.

// Public re-exports
pub use chain::Chain;
pub use pool_sync::PoolSync;
pub use pools::pool_structures::v3_structure::UniswapV3Pool;
pub use pools::{Pool, PoolInfo, PoolType};
pub use rpc::Rpc;

// Internal modules
mod builder;
mod cache;
mod chain;
mod errors;
mod events;
mod pool_sync;
mod pools;
mod rpc;
mod util;
mod tests;
