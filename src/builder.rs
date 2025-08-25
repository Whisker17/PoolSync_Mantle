//! PoolSync Builder Implementation
//!
//! This module provides a builder pattern for constructing a PoolSync instance,
//! allowing for flexible configuration of pool types and chains to be synced.

use crate::pools::pool_fetchers::{UniswapV3Fetcher, MerchantMoeV2Fetcher, AgniV3Fetcher};

use crate::errors::*;
use crate::pools::*;
use crate::{Chain, PoolSync, PoolType};
use std::collections::HashMap;
use std::sync::Arc;

/// Builder for constructing a PoolSync instance
#[derive(Default)]
pub struct PoolSyncBuilder {
    /// Mapping from the pool type to the implementation of its fetcher
    fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
    /// The chain to be synced on
    chain: Option<Chain>,
    /// Rate limit on the rpc endpoint
    rate_limit: Option<usize>,
    /// Optional starting block for synchronization
    start_block: Option<u64>,
    /// Optional ending block for synchronization  
    end_block: Option<u64>,
}

impl PoolSyncBuilder {
    /// Adds a new pool type to be synced
    /// The builder instance for method chaining
    pub fn add_pool(mut self, pool_type: PoolType) -> Self {
        match pool_type {
            PoolType::UniswapV3 => {
                self.fetchers
                    .insert(PoolType::UniswapV3, Arc::new(UniswapV3Fetcher));
            }
            PoolType::MerchantMoe => {
                self.fetchers
                    .insert(PoolType::MerchantMoe, Arc::new(MerchantMoeV2Fetcher));
            }
            PoolType::Agni => {
                self.fetchers
                    .insert(PoolType::Agni, Arc::new(AgniV3Fetcher));
            }
        }
        self
    }

    /// Add multiple pools to be synced
    pub fn add_pools(mut self, pools: &[PoolType]) -> Self {
        for pool in pools.iter() {
            self = self.add_pool(*pool);
        }
        self
    }

    /// Sets the chain to sync on
    /// The builder instance for method chaining
    pub fn chain(mut self, chain: Chain) -> Self {
        self.chain = Some(chain);
        self
    }

    /// Set the rate limit of the rpc
    /// The builder instance for method chaining
    pub fn rate_limit(mut self, rate_limit: usize) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Set the starting block for synchronization
    /// The builder instance for method chaining
    pub fn start_block(mut self, start_block: u64) -> Self {
        self.start_block = Some(start_block);
        self
    }

    /// Set the ending block for synchronization
    /// The builder instance for method chaining  
    pub fn end_block(mut self, end_block: u64) -> Self {
        self.end_block = Some(end_block);
        self
    }

    /// Set both start and end blocks for synchronization
    /// The builder instance for method chaining
    pub fn block_range(mut self, start_block: u64, end_block: u64) -> Self {
        self.start_block = Some(start_block);
        self.end_block = Some(end_block);
        self
    }

    /// Consumes the builder and produces a constructed PoolSync
    pub fn build(self) -> Result<PoolSync, PoolSyncError> {
        // Ensure the chain is set
        let chain = self.chain.ok_or(PoolSyncError::ChainNotSet)?;

        // Ensure all the pools are supported
        for pool_type in self.fetchers.keys() {
            if !chain.supported(pool_type) {
                return Err(PoolSyncError::UnsupportedPoolType);
            }
        }

        // set rate limit to user defined if specified, otherwise set high value
        // that will not be hit to simulate unlimited requests
        let rate_limit = self.rate_limit.unwrap_or(10000) as u64;

        // Construct PoolSync
        Ok(PoolSync {
            fetchers: self.fetchers,
            rate_limit,
            chain,
            start_block: self.start_block,
            end_block: self.end_block,
        })
    }
}