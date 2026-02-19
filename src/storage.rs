use soroban_sdk::{Address, Env};

use crate::errors::ContractError;
use crate::types::Trade;

const INITIALIZED: &str = "INIT";
const ADMIN: &str = "ADMIN";
const USDC_TOKEN: &str = "USDC";
const FEE_BPS: &str = "FEE_BPS";
const TRADE_COUNTER: &str = "COUNTER";
const ACCUMULATED_FEES: &str = "ACC_FEES";

const TRADE_PREFIX: &str = "TRADE";
const ARBITRATOR_PREFIX: &str = "ARB";

// Initialization
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&INITIALIZED)
}

pub fn has_initialized(env: &Env) -> bool {
    env.storage().instance().has(&INITIALIZED)
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&INITIALIZED, &true);
}

// Admin
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN, admin);
}

pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&ADMIN)
        .ok_or(ContractError::NotInitialized)
}

// USDC Token
pub fn set_usdc_token(env: &Env, token: &Address) {
    env.storage().instance().set(&USDC_TOKEN, token);
}

pub fn get_usdc_token(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&USDC_TOKEN)
        .ok_or(ContractError::NotInitialized)
}

// Fee BPS
pub fn set_fee_bps(env: &Env, fee_bps: u32) {
    env.storage().instance().set(&FEE_BPS, &fee_bps);
}

pub fn get_fee_bps(env: &Env) -> Result<u32, ContractError> {
    env.storage()
        .instance()
        .get(&FEE_BPS)
        .ok_or(ContractError::NotInitialized)
}

// Trade Counter
pub fn set_trade_counter(env: &Env, counter: u64) {
    env.storage().instance().set(&TRADE_COUNTER, &counter);
}

pub fn get_trade_counter(env: &Env) -> Result<u64, ContractError> {
    env.storage()
        .instance()
        .get(&TRADE_COUNTER)
        .ok_or(ContractError::NotInitialized)
}

pub fn increment_trade_counter(env: &Env) -> Result<u64, ContractError> {
    let current = get_trade_counter(env)?;
    let next = current.checked_add(1).ok_or(ContractError::Overflow)?;
    set_trade_counter(env, next);
    Ok(next)
}

// Accumulated Fees
pub fn set_accumulated_fees(env: &Env, fees: u64) {
    env.storage().instance().set(&ACCUMULATED_FEES, &fees);
}

pub fn get_accumulated_fees(env: &Env) -> Result<u64, ContractError> {
    env.storage()
        .instance()
        .get(&ACCUMULATED_FEES)
        .ok_or(ContractError::NotInitialized)
}

// Trades
pub fn save_trade(env: &Env, trade_id: u64, trade: &Trade) {
    let key = (TRADE_PREFIX, trade_id);
    env.storage().persistent().set(&key, trade);
}

pub fn get_trade(env: &Env, trade_id: u64) -> Result<Trade, ContractError> {
    let key = (TRADE_PREFIX, trade_id);
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::TradeNotFound)
}

// Arbitrators
pub fn save_arbitrator(env: &Env, arbitrator: &Address) {
    let key = (ARBITRATOR_PREFIX, arbitrator);
    env.storage().persistent().set(&key, &true);
}

pub fn remove_arbitrator(env: &Env, arbitrator: &Address) {
    let key = (ARBITRATOR_PREFIX, arbitrator);
    env.storage().persistent().remove(&key);
}

pub fn has_arbitrator(env: &Env, arbitrator: &Address) -> bool {
    let key = (ARBITRATOR_PREFIX, arbitrator);
    env.storage().persistent().has(&key)
}
