use alloy::dyn_abi::DynSolValue;
use alloy::primitives::{Address, U256};
use alloy::rpc::types::Log;
use alloy::sol_types::SolEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::events::DataEvents;
use crate::pools::PoolType;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UniswapV3Pool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub token0_name: String,
    pub token1_name: String,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
    pub liquidity: u128,
    pub sqrt_price: U256,
    pub fee: u32,
    pub tick: i32,
    pub tick_spacing: i32,
    pub tick_bitmap: HashMap<i16, U256>,
    pub ticks: HashMap<i32, TickInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TickInfo {
    pub liquidity_net: i128,
    pub initialized: bool,
    pub liquidity_gross: u128,
}

pub fn process_tick_data(
    pool: &mut UniswapV3Pool,
    log: Log,
    _pool_type: PoolType,
    is_initial_sync: bool,
) {
    let event_sig = log.topic0().unwrap();

    if *event_sig == DataEvents::Burn::SIGNATURE_HASH {
        process_burn(pool, log, is_initial_sync);
    } else if *event_sig == DataEvents::Mint::SIGNATURE_HASH {
        process_mint(pool, log, is_initial_sync);
    } else if *event_sig == DataEvents::Swap::SIGNATURE_HASH {
        process_swap(pool, log);
    }
}

fn process_burn(pool: &mut UniswapV3Pool, log: Log, is_initial_sync: bool) {
    let burn_event = DataEvents::Burn::decode_log(log.as_ref(), true).unwrap();
    modify_position(
        pool,
        burn_event.tickLower.unchecked_into(),
        burn_event.tickUpper.unchecked_into(),
        -(burn_event.amount as i128),
        is_initial_sync,
    );
}

fn process_mint(pool: &mut UniswapV3Pool, log: Log, is_initial_sync: bool) {
    let mint_event = DataEvents::Mint::decode_log(log.as_ref(), true).unwrap();
    modify_position(
        pool,
        mint_event.tickLower.unchecked_into(),
        mint_event.tickUpper.unchecked_into(),
        mint_event.amount as i128,
        is_initial_sync,
    );
}

fn process_swap(pool: &mut UniswapV3Pool, log: Log) {
    let swap_event = DataEvents::Swap::decode_log(log.as_ref(), true).unwrap();
    pool.tick = swap_event.tick.as_i32();
    pool.sqrt_price = U256::from(swap_event.sqrtPriceX96);
    pool.liquidity = swap_event.liquidity;
}

/// Modifies a positions liquidity in the pool.
pub fn modify_position(
    pool: &mut UniswapV3Pool,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
    is_initial_sync: bool,
) {
    //We are only using this function when a mint or burn event is emitted,
    //therefore we do not need to checkTicks as that has happened before the event is emitted
    update_position(pool, tick_lower, tick_upper, liquidity_delta);

    // if it is the initial sync, ignore since liq is populated via contract
    if liquidity_delta != 0 && !is_initial_sync {
        //if the tick is between the tick lower and tick upper, update the liquidity between the ticks
        if pool.tick >= tick_lower && pool.tick < tick_upper {
            pool.liquidity = if liquidity_delta < 0 {
                pool.liquidity - ((-liquidity_delta) as u128)
            } else {
                pool.liquidity + (liquidity_delta as u128)
            }
        }
    }
}

pub fn update_position(
    pool: &mut UniswapV3Pool,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
) {
    let mut flipped_lower = false;
    let mut flipped_upper = false;

    if liquidity_delta != 0 {
        flipped_lower = update_tick(pool, tick_lower, liquidity_delta, false);
        flipped_upper = update_tick(pool, tick_upper, liquidity_delta, true);
        if flipped_lower {
            flip_tick(pool, tick_lower, pool.tick_spacing);
        }
        if flipped_upper {
            flip_tick(pool, tick_upper, pool.tick_spacing);
        }
    }

    if liquidity_delta < 0 {
        if flipped_lower {
            pool.ticks.remove(&tick_lower);
        }

        if flipped_upper {
            pool.ticks.remove(&tick_upper);
        }
    }
}

pub fn update_tick(
    pool: &mut UniswapV3Pool,
    tick: i32,
    liquidity_delta: i128,
    upper: bool,
) -> bool {
    let info = match pool.ticks.get_mut(&tick) {
        Some(info) => info,
        None => {
            pool.ticks.insert(tick, TickInfo::default());
            pool.ticks
                .get_mut(&tick)
                .expect("Tick does not exist in ticks")
        }
    };

    let liquidity_gross_before = info.liquidity_gross;

    let liquidity_gross_after = if liquidity_delta < 0 {
        liquidity_gross_before - ((-liquidity_delta) as u128)
    } else {
        liquidity_gross_before + (liquidity_delta as u128)
    };

    // we do not need to check if liqudity_gross_after > maxLiquidity because we are only calling update tick on a burn or mint log.
    // this should already be validated when a log is
    let flipped = (liquidity_gross_after == 0) != (liquidity_gross_before == 0);

    if liquidity_gross_before == 0 {
        info.initialized = true;
    }

    info.liquidity_gross = liquidity_gross_after;

    info.liquidity_net = if upper {
        info.liquidity_net - liquidity_delta
    } else {
        info.liquidity_net + liquidity_delta
    };

    flipped
}

pub fn flip_tick(pool: &mut UniswapV3Pool, tick: i32, tick_spacing: i32) {
    let (word_pos, bit_pos) = uniswap_v3_math::tick_bitmap::position(tick / tick_spacing);
    let mask = U256::from(1) << bit_pos;

    if let Some(word) = pool.tick_bitmap.get_mut(&word_pos) {
        *word ^= mask;
    } else {
        pool.tick_bitmap.insert(word_pos, mask);
    }
}

impl From<&[DynSolValue]> for UniswapV3Pool {
    fn from(data: &[DynSolValue]) -> Self {
        // Safe conversion function for decimals with bounds checking
        let safe_u8_conversion = |value: &DynSolValue| -> u8 {
            let uint_val = value.as_uint().unwrap().0;
            if uint_val > U256::from(255u32) {
                18 // Default to 18 decimals if value is too large
            } else {
                uint_val.try_into().unwrap_or(18)
            }
        };

        // Safe conversion function for fee with bounds checking
        let safe_u32_conversion = |value: &DynSolValue| -> u32 {
            let uint_val = value.as_uint().unwrap().0;
            if uint_val > U256::from(u32::MAX) {
                3000 // Default fee if value is too large
            } else {
                uint_val.try_into().unwrap_or(3000)
            }
        };

        // Safe conversion function for tick values
        let safe_i32_conversion = |value: &DynSolValue| -> i32 {
            match value.as_int() {
                Some(signed_val) => {
                    // Try to convert to i32, use default if overflow
                    signed_val.0.try_into().unwrap_or(0)
                }
                None => 0, // Default to 0 if conversion fails
            }
        };

        Self {
            address: data[0].as_address().unwrap(),
            token0: data[1].as_address().unwrap(),
            token0_decimals: safe_u8_conversion(&data[2]),
            token1: data[3].as_address().unwrap(),
            token1_decimals: safe_u8_conversion(&data[4]),
            liquidity: data[5].as_uint().unwrap().0.try_into().unwrap_or(0),
            sqrt_price: data[6].as_uint().unwrap().0,
            tick: safe_i32_conversion(&data[7]),
            tick_spacing: safe_i32_conversion(&data[8]),
            fee: safe_u32_conversion(&data[9]),
            ..Default::default()
        }
    }
}
