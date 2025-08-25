use alloy::primitives::{address, Address};
use alloy::sol_types::SolEvent;
use alloy::primitives::Log;
use alloy::dyn_abi::DynSolType;
use crate::pools::PoolFetcher;
use crate::pools::gen::AgniV3Factory;
use crate::pools::PoolType;
use crate::Chain;

pub struct AgniV3Fetcher;

impl PoolFetcher for AgniV3Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::Agni
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Mantle => address!("25780dc8Fc3cfBD75F33bFDAB65e969b603b2035 "), // Agni V3 Factory on Mantle
        }
    }

    fn pair_created_signature(&self) -> &str {
        AgniV3Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = AgniV3Factory::PoolCreated::decode_log(log, false).unwrap();
        decoded_log.data.pool
    }

    fn get_pool_repr(&self) -> DynSolType {
        DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,     // pool address
            DynSolType::Address,     // token0
            DynSolType::Uint(8),     // token0 decimals  
            DynSolType::Address,     // token1
            DynSolType::Uint(8),     // token1 decimals
            DynSolType::Uint(128),   // liquidity
            DynSolType::Uint(160),   // sqrtPriceX96
            DynSolType::Int(24),     // tick
            DynSolType::Int(24),     // tickSpacing
            DynSolType::Uint(24),    // fee
        ])))
    }
}
