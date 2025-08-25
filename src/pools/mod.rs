//! Core definitions for pool synchronization
//!
//! This module defines the core structures and traits used in the pool synchronization system.
//! It includes enumerations for supported pool types, a unified `Pool` enum, and a trait for
//! fetching and decoding pool creation events.

use alloy::dyn_abi::DynSolType;
use alloy::dyn_abi::DynSolValue;
use alloy::primitives::{Address, Log};
use pool_structures::v3_structure::UniswapV3Pool;
use pool_structures::v2_structure::MerchantMoeV2Pool;

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::chain::Chain;
use crate::impl_pool_info;

mod gen;
pub mod pool_builder;
pub mod pool_fetchers;
pub mod pool_structures;

/// Enumerates the supported pool types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoolType {
    UniswapV3,
    MerchantMoe,
    Agni,
}

impl PoolType {
    pub fn is_v3(&self) -> bool {
        matches!(self, PoolType::UniswapV3 | PoolType::Agni)
    }
    
    pub fn is_v2(&self) -> bool {
        matches!(self, PoolType::MerchantMoe)
    }

    pub fn build_pool(&self, pool_data: &[DynSolValue]) -> Pool {
        if self.is_v3() {
            let pool = UniswapV3Pool::from(pool_data);
            Pool::new_v3(*self, pool)
        } else if self.is_v2() {
            let pool = MerchantMoeV2Pool::from(pool_data);
            Pool::new_v2(*self, pool)
        } else {
            panic!("Invalid pool type");
        }
    }
}

/// Represents a populated pool from any of the supported protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pool {
    UniswapV3(UniswapV3Pool),
    MerchantMoe(MerchantMoeV2Pool),
    Agni(UniswapV3Pool),
}

impl Pool {
    pub fn new_v3(pool_type: PoolType, pool: UniswapV3Pool) -> Self {
        match pool_type {
            PoolType::UniswapV3 => Pool::UniswapV3(pool),
            PoolType::Agni => Pool::Agni(pool),
            _ => panic!("Invalid pool type for V3"),
        }
    }
    
    pub fn new_v2(pool_type: PoolType, pool: MerchantMoeV2Pool) -> Self {
        match pool_type {
            PoolType::MerchantMoe => Pool::MerchantMoe(pool),
            _ => panic!("Invalid pool type for V2"),
        }
    }

    pub fn is_v3(&self) -> bool {
        matches!(self, Pool::UniswapV3(_) | Pool::Agni(_))
    }
    
    pub fn is_v2(&self) -> bool {
        matches!(self, Pool::MerchantMoe(_))
    }

    pub fn get_v3(&self) -> Option<&UniswapV3Pool> {
        match self {
            Pool::UniswapV3(pool) | Pool::Agni(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_v3_mut(&mut self) -> Option<&mut UniswapV3Pool> {
        match self {
            Pool::UniswapV3(pool) | Pool::Agni(pool) => Some(pool),
            _ => None,
        }
    }
    
    pub fn get_v2(&self) -> Option<&MerchantMoeV2Pool> {
        match self {
            Pool::MerchantMoe(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_v2_mut(&mut self) -> Option<&mut MerchantMoeV2Pool> {
        match self {
            Pool::MerchantMoe(pool) => Some(pool),
            _ => None,
        }
    }



    pub fn is_valid(&self) -> bool {
        self.address() != Address::ZERO
            && self.token0_address() != Address::ZERO
            && self.token1_address() != Address::ZERO
    }

    fn update_token0_name(pool: &mut Pool, token0: String) {
        if let Some(pool) = pool.get_v3_mut() {
            pool.token0_name = token0;
        } else if let Some(pool) = pool.get_v2_mut() {
            pool.token0_name = token0;
        }
    }

    pub fn update_token1_name(pool: &mut Pool, token1: String) {
        if let Some(pool) = pool.get_v3_mut() {
            pool.token1_name = token1;
        } else if let Some(pool) = pool.get_v2_mut() {
            pool.token1_name = token1;
        }
    }
}

impl fmt::Display for PoolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Implement the PoolInfo trait for all pool variants that are supported
impl_pool_info!(
    Pool,
    UniswapV3,
    MerchantMoe,
    Agni
);

/// Defines common functionality for fetching and decoding pool creation events
///
/// This trait provides a unified interface for different pool types to implement
/// their specific logic for identifying and parsing pool creation events.
pub trait PoolFetcher: Send + Sync {
    /// Returns the type of pool this fetcher is responsible for
    fn pool_type(&self) -> PoolType;

    /// Returns the factory address for the given chain
    fn factory_address(&self, chain: Chain) -> Address;

    /// Returns the event signature for pool creation
    fn pair_created_signature(&self) -> &str;

    /// Attempts to create a `Pool` instance from a log entry
    fn log_to_address(&self, log: &Log) -> Address;

    /// Get the DynSolType for the pool
    fn get_pool_repr(&self) -> DynSolType;
}

/// Defines common methods that are used to access information about the pools
pub trait PoolInfo {
    fn address(&self) -> Address;
    fn token0_address(&self) -> Address;
    fn token1_address(&self) -> Address;
    fn token0_name(&self) -> String;
    fn token1_name(&self) -> String;
    fn token0_decimals(&self) -> u8;
    fn token1_decimals(&self) -> u8;
    fn pool_type(&self) -> PoolType;
    fn fee(&self) -> u32;
    fn stable(&self) -> bool;
}

/* 
pub trait V2PoolInfo {
    fn token0_reserves(&self) -> U128;
    fn token1_reserves(&self) -> U128;
}

pub trait V3PoolInfo {
    fn fee(&self) -> u32;
    fn tick_spacing(&self) -> i32;
    fn tick_bitmap(&self) -> HashMap<i16, U256>;
    fn ticks(&self) -> HashMap<i32, TickInfo>;
}
*/

/// Macro for generating getter methods for all of the suppored pools
#[macro_export]
macro_rules! impl_pool_info {
    ($enum_name:ident, $($variant:ident),+) => {
        impl PoolInfo for $enum_name {
            fn address(&self) -> Address {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.address,

                    )+
                }
            }

            fn token0_address(&self) -> Address {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token0,
                    )+
                }
            }

            fn token1_address(&self) -> Address {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token1,
                    )+
                }
            }

            fn token0_name(&self) -> String {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token0_name.clone(),
                    )+
                }
            }
            fn token1_name(&self) -> String {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token1_name.clone(),
                    )+
                }
            }

            fn token0_decimals(&self) -> u8 {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token0_decimals,
                    )+
                }
            }
            fn token1_decimals(&self) -> u8 {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token1_decimals,
                    )+
                }
            }

            fn pool_type(&self) -> PoolType {
                match self {
                    $(
                        $enum_name::$variant(_) => PoolType::$variant,
                    )+
                }
            }

            fn fee(&self) -> u32 {
                match self {
                    Pool::UniswapV3(pool) | Pool::Agni(pool) => pool.fee,
                    Pool::MerchantMoe(_) => 0, // V2 pools don't have fees in the same way
                }
            }

            fn stable(&self) -> bool {
                false
            }
        }
    };
}
