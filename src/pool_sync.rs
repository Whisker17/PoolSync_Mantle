//! PoolSync Core Implementation
//!
//! This module contains the core functionality for synchronizing pools across different
//! blockchain networks and protocols. It includes the main `PoolSync` struct and its
//! associated methods for configuring and executing the synchronization process.
//!
use alloy::providers::Provider;
use alloy::providers::ProviderBuilder;
use std::collections::HashMap;
use std::sync::Arc;

use crate::builder::PoolSyncBuilder;
use crate::cache::{read_cache_file, write_cache_file, PoolCache};
use crate::chain::Chain;
use crate::errors::*;
use crate::pools::*;
use crate::rpc::Rpc;

/// The main struct for pool synchronization
pub struct PoolSync {
    /// Map of pool types to their fetcher implementations
    pub fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
    /// The chain to sync on
    pub chain: Chain,
    /// The rate limit of the rpc
    pub rate_limit: u64,
    /// Optional starting block for synchronization (overrides cache)
    pub start_block: Option<u64>,
    /// Optional ending block for synchronization (overrides latest block)
    pub end_block: Option<u64>,
}

impl PoolSync {
    /// Construct a new builder to configure sync parameters
    pub fn builder() -> PoolSyncBuilder {
        PoolSyncBuilder::default()
    }

    /// Synchronizes all added pools for the specified chain
    pub async fn sync_pools(&self) -> Result<(Vec<Pool>, u64), PoolSyncError> {
        // load in the dotenv
        dotenv::dotenv().ok();

        // setup arvhice node provider
        let archive = Arc::new(
            ProviderBuilder::new()
                .network::<alloy::network::AnyNetwork>()
                .on_http(std::env::var("ARCHIVE").unwrap().parse().unwrap()),
        );

        // setup full node provider
        let full = Arc::new(
            ProviderBuilder::new()
                .network::<alloy::network::AnyNetwork>()
                .on_http(std::env::var("FULL").unwrap().parse().unwrap()),
        );

        // create the cache files
        std::fs::create_dir_all("cache").unwrap();

        // create all of the caches
        let mut pool_caches: Vec<PoolCache> = self
            .fetchers
            .keys()
            .map(|pool_type| read_cache_file(pool_type, self.chain).unwrap())
            .collect();

        let mut fully_synced = false;
        let mut last_synced_block = 0;

        while !fully_synced {
            fully_synced = true;
            
            // Use custom end_block if specified, otherwise get latest block
            let end_block = match self.end_block {
                Some(end_block) => end_block,
                None => full.get_block_number().await.unwrap(),
            };

            println!("\n🔄 开始同步轮次 - 目标区块: {}, 上次同步: {}", end_block, last_synced_block);
            println!("📊 协议状态:");
            for cache in &pool_caches {
                println!("  {} - 缓存池数: {}, 上次同步区块: {}", 
                    cache.pool_type, cache.pools.len(), cache.last_synced_block);
            }
            println!("");

            for cache in &mut pool_caches {
                // Use custom start_block if specified, otherwise use cache
                let start_block = match self.start_block {
                    Some(start_block) => {
                        // 如果指定了自定义起始区块，只有在缓存还没达到这个区块时才使用
                        if cache.last_synced_block < start_block {
                            start_block
                        } else {
                            cache.last_synced_block + 1
                        }
                    },
                    None => cache.last_synced_block + 1,
                };
                
                if start_block <= end_block {
                    fully_synced = false;
                    
                    println!("🔗 正在同步 {} 协议 (区块 {} → {})", cache.pool_type, start_block, end_block);

                    let fetcher = self.fetchers[&cache.pool_type].clone();

                    // fetch all of the pool addresses
                    let pool_addrs = Rpc::fetch_pool_addrs(
                        start_block,
                        end_block,
                        archive.clone(),
                        fetcher.clone(),
                        self.chain,
                        self.rate_limit,
                    )
                    .await
                    .expect(
                        "Failed to fetch pool addresses. Exiting due to having inconclusive state",
                    );

                    // populate all of the pool data
                    let mut new_pools = Rpc::populate_pools(
                        pool_addrs,
                        full.clone(),
                        cache.pool_type,
                        fetcher.clone(),
                        self.rate_limit,
                        self.chain,
                    )
                    .await
                    .expect("Failed to sync pool data, Exiting due to haveing inconclusive state");


                    // catch up all the old pools
                    Rpc::populate_liquidity(
                        start_block,
                        end_block,
                        &mut cache.pools,
                        archive.clone(),
                        cache.pool_type,
                        self.rate_limit,
                        cache.is_initial_sync,
                    )
                    .await
                    .expect("Failed to populate liquidity information, Exiting due to having inconclusive state");

                    // update the new pools
                    if !new_pools.is_empty() {
                        Rpc::populate_liquidity(
                            start_block,
                            end_block,
                            &mut new_pools,
                            archive.clone(),
                            cache.pool_type,
                            self.rate_limit,
                            true,
                        )
                        .await
                        .expect("Failed to populate liquidity information, Exiting due to having inconclusive state");
                    }


                    // merge old and new
                    let new_pools_count = new_pools.len();
                    cache.pools.extend(new_pools);


                    // update info for cache
                    cache.last_synced_block = end_block;
                    last_synced_block = end_block;
                    cache.is_initial_sync = false;
                    
                    println!("✅ {} 协议同步完成 - 总池数: {}, 新增池: {}, 同步至区块: {}", 
                        cache.pool_type, cache.pools.len(), new_pools_count, end_block);
                } else {
                    println!("⏭️  {} 协议已为最新状态 (区块 {})", cache.pool_type, cache.last_synced_block);
                }
            }
            
            // 如果指定了自定义的end_block，检查是否所有协议都已同步完成
            if let Some(target_end_block) = self.end_block {
                let all_synced_to_target = pool_caches.iter().all(|cache| cache.last_synced_block >= target_end_block);
                if all_synced_to_target {
                    println!("🎯 所有协议已同步至目标区块 {}, 同步完成!", target_end_block);
                    break;
                }
            }
        }

        println!("\n🎉 所有协议同步完成! 最终状态:");
        for cache in &pool_caches {
            println!("  {} - 总池数: {}, 最新区块: {}", 
                cache.pool_type, cache.pools.len(), cache.last_synced_block);
        }
        println!("💾 正在保存缓存文件...\n");

        // write all of the cache files
        pool_caches
            .iter()
            .for_each(|cache| write_cache_file(cache, self.chain).unwrap());

        // return all the pools
        Ok((
            pool_caches
                .into_iter()
                .flat_map(|cache| cache.pools)
                .collect(),
            last_synced_block,
        ))
    }
}

