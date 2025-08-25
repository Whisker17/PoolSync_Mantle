# PoolSync For Mantle

PoolSync is a comprehensive utility crate for efficiently synchronizing DeFi pools from multiple protocols on the Mantle blockchain. This crate streamlines the process of pool synchronization with support for both V2 and V3 style AMMs, intelligent caching, and rate limiting, eliminating the need for repetitive boilerplate code in DeFi projects targeting the Mantle ecosystem.

## Features

- 🏊‍♂️ **Multi-Protocol Support** - UniswapV3, Agni (V3-style), and MerchantMoe (V2-style)
- 🚀 **Smart Caching System** - Persistent cache for each protocol with automatic management
- ⚡ **Dual-Node Architecture** - Archive + Full node setup for optimal performance and cost efficiency
- 🔄 **Rate Limiting** - Built-in rate limiting for public endpoints
- 📦 **Flexible Configuration** - Builder pattern with block range support
- 🔍 **Automatic Token Resolution** - Fetches token names and metadata automatically
- 🛠️ **Extensible Architecture** - Easy to add new protocols with trait-based design
- 📊 **Progress Tracking** - Built-in logging and progress indicators

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pool-sync = "3.0.0"
```

Configure your `.env` with both a full node and an archive node. The archive endpoint must be an archive node, while the full node can be either type. This dual-node design optimizes costs - use a paid archive endpoint for the initial intensive sync, then let the full node handle ongoing synchronization. After initial sync, all data is cached locally, dramatically reducing endpoint strain.

```env
FULL = "full node endpoint"
ARCHIVE = "archive node endpoint"
```

## Supported Protocols

### Mantle Network
- **Uniswap V3** - Advanced AMM with concentrated liquidity and tick-based pricing
- **Agni Finance** - V3-style AMM (Uniswap V3 fork) with concentrated liquidity
- **Merchant Moe** - V2-style constant product AMM with pair-based liquidity

## Example Usage

### Basic Pool Synchronization
```rust
use pool_sync::{PoolSync, PoolType, Chain, PoolInfo};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure and build the PoolSync instance for Mantle
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV3)     // Add Uniswap V3 pools
        .add_pool(PoolType::Agni)          // Add Agni pools  
        .add_pool(PoolType::MerchantMoe)   // Add MerchantMoe V2 pools
        .chain(Chain::Mantle)              // Target Mantle network
        .rate_limit(1000)                  // Set rate limit (ms between requests)
        .build()?;

    // Synchronize pools
    let (pools, last_synced_block) = pool_sync.sync_pools().await?;

    // Display pool information
    for pool in &pools {
        println!(
            "Pool: {} | Type: {:?} | Token0: {} | Token1: {} | Fee: {}",
            pool.address(),
            pool.pool_type(),
            pool.token0_name(),
            pool.token1_name(),
            pool.fee()
        );
    }

    println!("Successfully synced {} pools up to block {}!", pools.len(), last_synced_block);
    Ok(())
}
```

### Sync Specific Block Range
```rust
use pool_sync::{PoolSync, PoolType, Chain};

#[tokio::main]
async fn main() -> Result<()> {
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::Agni)
        .chain(Chain::Mantle)
        .rate_limit(500)
        .block_range(80000000, 80010000)  // Sync specific range
        .build()?;

    let (pools, _) = pool_sync.sync_pools().await?;
    println!("Synced {} Agni pools in specified range", pools.len());
    Ok(())
}
```

## Architecture

### Caching System
PoolSync uses an intelligent caching system that creates separate cache files for each protocol:
- `cache/Mantle_UniswapV3_cache.json`
- `cache/Mantle_Agni_cache.json` 
- `cache/Mantle_MerchantMoe_cache.json`

The cache stores the last synced block number and pool data, enabling efficient incremental updates on subsequent runs.

### Pool Structures
The library supports both V2 and V3 style pools through a unified interface:

- **V3 Pools** (UniswapV3, Agni): Concentrated liquidity with tick-based pricing
- **V2 Pools** (MerchantMoe): Constant product formula with pair-based liquidity

All pools implement the `PoolInfo` trait providing consistent access to:
- Pool and token addresses
- Token names, decimals, and symbols  
- Pool type identification
- Fee information
- Stability flags

### Modular Fetcher System
Each protocol implements the `PoolFetcher` trait, providing:
- Factory address for pool discovery
- Event signature for pool creation events
- Pool data parsing and structure creation
- Chain-specific configuration

## Advanced Usage

### Performance Optimization
```rust
use pool_sync::{PoolSync, PoolType, Chain};

#[tokio::main]
async fn main() -> Result<()> {
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV3)
        .chain(Chain::Mantle)
        .rate_limit(500)                    // Adjust based on endpoint limits
        .block_range(None, Some(80000000))  // Sync up to specific block
        .build()?;

    let (pools, last_block) = pool_sync.sync_pools().await?;
    
    // Filter by specific criteria
    let high_fee_pools: Vec<_> = pools.iter()
        .filter(|p| p.fee() >= 3000)  // 0.3% fee or higher
        .collect();
        
    println!("Found {} high-fee pools", high_fee_pools.len());
    Ok(())
}
```

### Working with Different Pool Types
```rust
use pool_sync::{Pool, PoolType};

fn analyze_pool(pool: &Pool) {
    match pool {
        Pool::UniswapV3(v3_pool) => {
            println!("V3 Pool: {} (tick spacing: {})", 
                     v3_pool.address, v3_pool.tick_spacing);
        },
        Pool::Agni(agni_pool) => {
            println!("Agni Pool: {} (fee: {})", 
                     agni_pool.address, agni_pool.fee);
        },
        Pool::MerchantMoe(v2_pool) => {
            println!("V2 Pool: {} (stable: {})", 
                     v2_pool.address, v2_pool.stable);
        }
    }
}
```

## Adding New Protocols

This codebase focuses specifically on Mantle network. To add a new DEX protocol:

### 1. **Add Pool ABI**
Add the factory ABI to `src/pools/abis/YourProtocol.json`

### 2. **Create Pool Fetcher**
Create `src/pools/pool_fetchers/yourprotocol/yourprotocol_fetcher.rs`:

```rust
use crate::pools::PoolFetcher;
use alloy::primitives::Address;

pub struct YourProtocolFetcher;

impl PoolFetcher for YourProtocolFetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::YourProtocol
    }
    
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Mantle => "your_factory_address".parse().unwrap(),
        }
    }
    
    // Implement other required methods...
}
```

### 3. **Update Pool Types**
Add your protocol to `src/pools/mod.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoolType {
    UniswapV3,
    MerchantMoe,
    Agni,
    YourProtocol,  // Add here
}
```

### 4. **Register in Builder**
Update `src/builder.rs` to include your fetcher

### 5. **Add to Chain Support**
Update `src/chain.rs` to map your protocol to Mantle

Refer to existing implementations (Agni, MerchantMoe) as examples.

## Troubleshooting

### Common Issues

**Cache Corruption**: Delete the cache files in `cache/` directory to force a full resync

**Rate Limiting**: Increase the rate limit value or upgrade to a paid RPC endpoint

**Memory Usage**: For large block ranges, consider syncing in smaller chunks

**Archive Node Requirement**: The initial sync requires an archive node for historical data access

## Roadmap

- [ ] **Enhanced Protocol Support** - Add more Mantle-native DEXs
- [ ] **Database Integration** - Optional PostgreSQL/SQLite backend
- [ ] **Real-time Updates** - WebSocket subscriptions for live pool updates  
- [ ] **Cross-chain Support** - Extend beyond Mantle to other networks
- [ ] **Pool Analytics** - Built-in TVL, volume, and liquidity analysis
- [ ] **Batch Operations** - Bulk token resolution and metadata fetching

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## Acknowledgments

Took inspiration from [amm-rs](https://github.com/darkforestry/amms-rs) - an excellent AMM library worth checking out!
