//! Pool builder for constructing pools from raw data

use crate::PoolInfo;
use alloy::dyn_abi::DynSolType;
use alloy::network::Network;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::transports::Transport;
use anyhow::Result;
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;

use super::gen::{V3DataSync, V2DataSync};

use crate::pools::gen::ERC20;
use crate::pools::{Pool, PoolType, Chain};

pub const INITIAL_BACKOFF: u64 = 1000; // 1 second
pub const MAX_RETRIES: u32 = 5;

pub async fn build_pools<P, T, N>(
    provider: &Arc<P>,
    addresses: Vec<Address>,
    pool_type: PoolType,
    data: DynSolType,
    chain: Chain,
) -> Result<Vec<Pool>>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let mut retry_count = 0;
    let mut backoff = INITIAL_BACKOFF;

    loop {
        match populate_pool_data(provider, addresses.clone(), pool_type, data.clone(), chain).await
        {
            Ok(pools) => {
                return Ok(pools);
            }
            Err(e) => {
                if retry_count >= MAX_RETRIES {
                    eprintln!("Max retries reached. Error: {:?} {:?}", e, addresses);
                    return Ok(Vec::new());
                }

                let jitter = rand::thread_rng().gen_range(0..=100);
                let sleep_duration = Duration::from_millis(backoff + jitter);
                tokio::time::sleep(sleep_duration).await;

                retry_count += 1;
                backoff *= 2; // Exponential backoff
            }
        }
    }
}

async fn populate_pool_data<P, T, N>(
    provider: &Arc<P>,
    pool_addresses: Vec<Address>,
    pool_type: PoolType,
    data: DynSolType,
    _chain: Chain
) -> Result<Vec<Pool>>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let pool_data = match pool_type {
        // V3-style pools (Uniswap V3, Agni)
        PoolType::UniswapV3 | PoolType::Agni => {
            V3DataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        }
        // V2-style pools (MerchantMoe)
        PoolType::MerchantMoe => {
            V2DataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        }
    };

    let decoded_data = data.abi_decode_sequence(&pool_data)?;
    let mut pools = Vec::new();

    if let Some(pool_data_arr) = decoded_data.as_array() {
        for pool_data_tuple in pool_data_arr {
            if let Some(pool_data) = pool_data_tuple.as_tuple() {
                let pool = pool_type.build_pool(pool_data);
                if pool.is_valid() {
                    pools.push(pool);
                }
            }
        }
    }

    // Fill in missing token names and symbols
    for pool in &mut pools {
        let token0_contract = ERC20::new(pool.token0_address(), &provider);
        if let Ok(ERC20::symbolReturn { _0: name }) = token0_contract.symbol().call().await {
            Pool::update_token0_name(pool, name);
        }

        let token1_contract = ERC20::new(pool.token1_address(), &provider);
        if let Ok(ERC20::symbolReturn { _0: name }) = token1_contract.symbol().call().await {
            Pool::update_token1_name(pool, name);
        }
    }

    Ok(pools)
}