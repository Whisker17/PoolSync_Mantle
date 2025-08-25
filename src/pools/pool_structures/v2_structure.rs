
use crate::events::{DataEvents};
use crate::pools::PoolType;
use alloy::dyn_abi::DynSolValue;
use alloy::primitives::{Address, U256};
use alloy::rpc::types::Log;
use alloy::sol_types::SolEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MerchantMoeV2Pool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub token0_name: String,
    pub token1_name: String,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
    pub token0_reserves: U256,
    pub token1_reserves: U256, 
}

pub fn process_sync_data(pool: &mut MerchantMoeV2Pool, log: Log, _pool_type: PoolType) {
    let sync_event = DataEvents::Sync::decode_log(log.as_ref(), true).unwrap();
    let (reserve0, reserve1) = (U256::from(sync_event.reserve0), U256::from(sync_event.reserve1));
    pool.token0_reserves = reserve0;
    pool.token1_reserves = reserve1;
}

impl From<&[DynSolValue]> for MerchantMoeV2Pool {
    fn from(data: &[DynSolValue]) -> Self {
        Self {
            address: data[0].as_address().unwrap(),
            token0: data[1].as_address().unwrap(),
            token1: data[2].as_address().unwrap(),
            token0_decimals: data[3].as_uint().unwrap().0.to::<u8>(),
            token1_decimals: data[4].as_uint().unwrap().0.to::<u8>(),
            token0_reserves: data[5].as_uint().unwrap().0.to::<U256>(),
            token1_reserves: data[6].as_uint().unwrap().0.to::<U256>(),
            ..Default::default()
        }
    }
}

