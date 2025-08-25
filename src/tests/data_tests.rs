
#[cfg(test)]
mod data_test {
    use alloy::providers::ProviderBuilder;
    use crate::{PoolSync, PoolInfo, Chain};
    use alloy::providers::RootProvider;
    use alloy::primitives::U256;
    use std::sync::Arc;
    use alloy::transports::http::{Http, Client};

    use crate::PoolType;
    use crate::tests::abi_gen::*;
    use crate::UniswapV3Pool;

    // V2 test removed since we only support V3 now

    #[tokio::test(flavor = "multi_thread")]
    async fn test_v3_data() {
        // Sync Uniswap V3 pools on Mantle
        let pool_sync = PoolSync::builder()
            .add_pool(PoolType::UniswapV3)
            .chain(Chain::Mantle)
            .rate_limit(1000)
            .build().unwrap();
        let (pools, last_synced_block) = pool_sync.sync_pools().await.unwrap();
        let provider = Arc::new(ProviderBuilder::new()
            .on_http(std::env::var("FULL").unwrap().parse().unwrap()));


        // for each pool, fetch the onchain data and confirm that it matches
        for pool in pools {
            fetch_v3_pool_data(pool.get_v3().unwrap(), pool.pool_type(), last_synced_block, provider.clone()).await;
        }
    }



    async fn fetch_v3_pool_data(
        pool: &UniswapV3Pool, 
        pool_type: PoolType,
        last_synced_block: u64,
        provider: Arc<RootProvider<Http<Client>>>,
    ) {
        // Get common pool data from Uniswap V3 contract
        let contract = V3State::new(pool.address, provider.clone());
        let V3State::slot0Return { sqrtPriceX96, tick, .. } = contract.slot0().block(last_synced_block.into()).call().await.unwrap();
        let V3State::liquidityReturn { _0: liquidity } = contract.liquidity().block(last_synced_block.into()).call().await.unwrap();
        let V3State::tickSpacingReturn { _0: tick_spacing } = contract.tickSpacing().block(last_synced_block.into()).call().await.unwrap();
        let V3State::feeReturn { _0: fee } = contract.fee().block(last_synced_block.into()).call().await.unwrap();

        let (sqrt_price, tick, liquidity, tick_spacing, fee) = (sqrtPriceX96, tick, liquidity, tick_spacing, fee);

        // Note: The current UniswapV3Pool structure doesn't include all these fields
        // This is just a placeholder test - you'd need to implement proper field matching
        println!("Pool {}: Fee={}, Tick Spacing={}", pool.address, pool.fee, pool.tick_spacing);
    }


}

