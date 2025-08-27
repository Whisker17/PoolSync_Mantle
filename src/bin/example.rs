//! Pool Synchronization Program
//!
//! This program synchronizes pools from a specified blockchain using the PoolSync library.
//! It demonstrates how to set up a provider, configure pool synchronization, and execute the sync process.
use anyhow::Result;
use pool_sync_mantle::{Chain, PoolSync, PoolType, PoolInfo};
use env_logger;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    // Example: Sync Uniswap V3 pools on Mantle
    let start_block = 80000400; // Starting block for Mantle
    let end_block = 80803400;   // Ending block (testing small range)
    
    println!("Syncing Agni pools on Mantle from block {} to {}...", start_block, end_block);
    
    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::Agni)
        .chain(Chain::Mantle)
        .rate_limit(1000)
        .block_range(start_block, end_block)  // 设置区块范围
        .build()?;

    // Synchronize pools
    let (pools, last_synced_block) = pool_sync.sync_pools().await?;
    println!(
        "Sync completed! Synced {} pools, last synced block: {}",
        pools.len(),
        last_synced_block
    );

    // Display information about some pools
    println!("\nFirst 5 pools information:");
    for (i, pool) in pools.iter().take(5).enumerate() {
        println!(
            "  {}. Address: {:?}, Token0: {}, Token1: {}", 
            i + 1,
            pool.address(), 
            pool.token0_name(), 
            pool.token1_name()
        );
    }

    Ok(())
}
