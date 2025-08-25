use alloy::primitives::{address, Address};
use alloy::sol_types::SolEvent;
use alloy::primitives::Log;
use alloy::dyn_abi::DynSolType;
use crate::pools::PoolFetcher;
use crate::pools::gen::MerchantMoeV2Factory;
use crate::pools::PoolType;
use crate::Chain;
pub struct MerchantMoeV2Fetcher;

impl PoolFetcher for MerchantMoeV2Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::MerchantMoe
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Mantle => address!("5bEf015CA9424A7C07B68490616a4C1F094BEdEc "),
        }
    }

    fn pair_created_signature(&self) -> &str {
        MerchantMoeV2Factory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = MerchantMoeV2Factory::PairCreated::decode_log(log, false).unwrap();
        decoded_log.data.pair
        
    }

    fn get_pool_repr(&self) -> DynSolType {
        DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(8),
            DynSolType::Uint(112),
            DynSolType::Uint(112),
        ])))
    }


}