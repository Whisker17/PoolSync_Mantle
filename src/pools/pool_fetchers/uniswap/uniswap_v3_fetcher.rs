use alloy::primitives::{address, Address};
use alloy::sol_types::SolEvent;
use alloy::primitives::Log;
use alloy::dyn_abi::DynSolType;
use crate::pools::PoolFetcher;
use crate::pools::gen::UniswapV3Factory;
use crate::pools::PoolType;
use crate::Chain;
pub struct UniswapV3Fetcher;

impl PoolFetcher for UniswapV3Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV3
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Mantle => address!("0d922Fb1Bc191F64970ac40376643808b4B74Df9"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        UniswapV3Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = UniswapV3Factory::PoolCreated::decode_log(log, false).unwrap();
        decoded_log.data.pool
        
    }

    fn get_pool_repr(&self) -> DynSolType {
        DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(128),
            DynSolType::Uint(160),
            DynSolType::Int(24),
            DynSolType::Int(24),
            DynSolType::Uint(24),
            DynSolType::Int(128),
        ])))
    }


}