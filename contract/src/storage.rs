use soroban_sdk::{contracttype, Address, Env, String};

use crate::{
    errors::ContractError,
    types::{CrossChainInfo, InsurancePolicy, Trade, UserCompliance},
};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    UsdcToken,
    FeeBps,
    Initialized,
    Paused,
    TradeCounter,
    AccumulatedFees,
    Version,
    GlobalTradeLimit,
    BridgeOracle,
    Trade(u64),
    Arbitrator(Address),
    UserCompliance(Address),
    UserTradeLimit(Address),
    JurisdictionRule(String),
    CrossChainInfo(u64),
    InsuranceProvider(Address),
    InsurancePolicy(u64),
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::Initialized).unwrap_or(false)
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&DataKey::Initialized, &true);
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(ContractError::NotInitialized)
}

pub fn set_usdc_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::UsdcToken, token);
}

pub fn get_usdc_token(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::UsdcToken)
        .ok_or(ContractError::NotInitialized)
}

pub fn set_fee_bps(env: &Env, fee_bps: u32) {
    env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
}

pub fn get_fee_bps(env: &Env) -> Result<u32, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::FeeBps)
        .ok_or(ContractError::NotInitialized)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
}

pub fn set_trade_counter(env: &Env, counter: u64) {
    env.storage().instance().set(&DataKey::TradeCounter, &counter);
}

pub fn increment_trade_counter(env: &Env) -> Result<u64, ContractError> {
    let next = env
        .storage()
        .instance()
        .get::<_, u64>(&DataKey::TradeCounter)
        .unwrap_or(0)
        .checked_add(1)
        .ok_or(ContractError::Overflow)?;
    env.storage().instance().set(&DataKey::TradeCounter, &next);
    Ok(next)
}

pub fn save_trade(env: &Env, trade_id: u64, trade: &Trade) {
    env.storage().persistent().set(&DataKey::Trade(trade_id), trade);
}

pub fn get_trade(env: &Env, trade_id: u64) -> Result<Trade, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::Trade(trade_id))
        .ok_or(ContractError::TradeNotFound)
}

pub fn save_arbitrator(env: &Env, arbitrator: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::Arbitrator(arbitrator.clone()), &true);
}

pub fn remove_arbitrator(env: &Env, arbitrator: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::Arbitrator(arbitrator.clone()));
}

pub fn has_arbitrator(env: &Env, arbitrator: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Arbitrator(arbitrator.clone()))
}

pub fn save_user_compliance(env: &Env, user: &Address, compliance: &UserCompliance) {
    env.storage()
        .persistent()
        .set(&DataKey::UserCompliance(user.clone()), compliance);
}

pub fn get_user_compliance(env: &Env, user: &Address) -> Option<UserCompliance> {
    env.storage()
        .persistent()
        .get(&DataKey::UserCompliance(user.clone()))
}

pub fn set_user_trade_limit(env: &Env, user: &Address, limit: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::UserTradeLimit(user.clone()), &limit);
}

pub fn get_user_trade_limit(env: &Env, user: &Address) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::UserTradeLimit(user.clone()))
        .unwrap_or(0)
}

pub fn set_jurisdiction_rule(env: &Env, jurisdiction: &String, allowed: bool) {
    env.storage()
        .persistent()
        .set(&DataKey::JurisdictionRule(jurisdiction.clone()), &allowed);
}

pub fn is_jurisdiction_allowed(env: &Env, jurisdiction: &String) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::JurisdictionRule(jurisdiction.clone()))
        .unwrap_or(true)
}

pub fn set_global_trade_limit(env: &Env, limit: u64) {
    env.storage().instance().set(&DataKey::GlobalTradeLimit, &limit);
}

pub fn get_global_trade_limit(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::GlobalTradeLimit)
        .unwrap_or(u64::MAX)
}

pub fn set_accumulated_fees(env: &Env, fees: u64) {
    env.storage().instance().set(&DataKey::AccumulatedFees, &fees);
}

pub fn get_accumulated_fees(env: &Env) -> Result<u64, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::AccumulatedFees)
        .ok_or(ContractError::NotInitialized)
}

pub fn add_accumulated_fees(env: &Env, delta: u64) -> Result<u64, ContractError> {
    let updated = env
        .storage()
        .instance()
        .get::<_, u64>(&DataKey::AccumulatedFees)
        .unwrap_or(0)
        .checked_add(delta)
        .ok_or(ContractError::Overflow)?;
    env.storage().instance().set(&DataKey::AccumulatedFees, &updated);
    Ok(updated)
}

pub fn set_version(env: &Env, version: u32) {
    env.storage().instance().set(&DataKey::Version, &version);
}

pub fn get_version(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Version).unwrap_or(1)
}

pub fn set_bridge_oracle(env: &Env, oracle: &Address) {
    env.storage().instance().set(&DataKey::BridgeOracle, oracle);
}

pub fn get_bridge_oracle(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::BridgeOracle)
}

pub fn save_cross_chain_info(env: &Env, trade_id: u64, info: &CrossChainInfo) {
    env.storage()
        .persistent()
        .set(&DataKey::CrossChainInfo(trade_id), info);
}

pub fn get_cross_chain_info(env: &Env, trade_id: u64) -> Option<CrossChainInfo> {
    env.storage()
        .persistent()
        .get(&DataKey::CrossChainInfo(trade_id))
}

pub fn save_insurance_provider(env: &Env, provider: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::InsuranceProvider(provider.clone()), &true);
}

pub fn remove_insurance_provider(env: &Env, provider: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::InsuranceProvider(provider.clone()));
}

pub fn has_insurance_provider(env: &Env, provider: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::InsuranceProvider(provider.clone()))
}

pub fn save_insurance_policy(env: &Env, trade_id: u64, policy: &InsurancePolicy) {
    env.storage()
        .persistent()
        .set(&DataKey::InsurancePolicy(trade_id), policy);
}

pub fn get_insurance_policy(env: &Env, trade_id: u64) -> Option<InsurancePolicy> {
    env.storage()
        .persistent()
        .get(&DataKey::InsurancePolicy(trade_id))
}
