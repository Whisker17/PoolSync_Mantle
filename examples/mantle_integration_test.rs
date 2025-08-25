//! Mantle Integration Test Example
//!
//! This example demonstrates how to use PoolSync with Mantle chain.
//! It follows the same pattern as your provided example but targets Mantle.

use anyhow::Result;
use pool_sync::{Chain, PoolInfo, PoolSync, PoolType};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure and build the PoolSync instance for Mantle
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV3)      // Uniswap V3 on Mantle
        .chain(Chain::Mantle)               // Target Mantle chain
        .build()?;

    // Synchronize pools
    let (pools, last_synced_block) = pool_sync.sync_pools().await?;

    // Display results (same format as your example)
    println!("=== Mantle Chain Pool Sync Results ===");
    for pool in &pools {
        println!(
            "Pool Address {:?}, Token 0: {:?}, Token 1: {:?}",
            pool.address(),
            pool.token0_name(),
            pool.token1_name()
        );
    }

    println!("Synced {} pools on Mantle chain (Uniswap V3)!", pools.len());
    println!("Last synced block: {}", last_synced_block);

    Ok(())
}


