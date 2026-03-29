//! Partial Release Helper Functions
//! 
//! This module provides helper functions for validating and calculating
//! partial release amounts during dispute resolution.

use soroban_sdk::Env;
use crate::errors::ContractError;
use crate::storage::get_trade;

/// Calculate partial release amounts for a dispute resolution.
/// 
/// Returns a tuple of (buyer_amount, seller_amount) based on the buyer_bps parameter.
/// 
/// # Arguments
/// * `env` - The Soroban environment
/// * `trade_id` - The trade ID to calculate amounts for
/// * `buyer_bps` - The buyer's share in basis points (0-10000)
/// 
/// # Returns
/// * `Ok((buyer_amount, seller_amount))` - The calculated amounts
/// * `Err(ContractError)` - If validation fails
/// 
/// # Example
/// ```
/// let (buyer_amt, seller_amt) = calculate_partial_amounts(&env, trade_id, 6000)?;
/// // buyer receives 60%, seller receives 40%
/// ```
pub fn calculate_partial_amounts(
    env: &Env,
    trade_id: u64,
    buyer_bps: u32,
) -> Result<(u64, u64), ContractError> {
    // Validate buyer_bps is within valid range
    if buyer_bps > 10000 {
        return Err(ContractError::InvalidSplitBps);
    }
    
    // Get trade details
    let trade = get_trade(env, trade_id)?;
    
    // Calculate net amount after fee
    let net = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
    
    // Calculate buyer amount: net * buyer_bps / 10000
    let buyer_amount = net
        .checked_mul(buyer_bps as u64)
        .ok_or(ContractError::Overflow)?
        .checked_div(10000)
        .ok_or(ContractError::Overflow)?;
    
    // Calculate seller amount: net - buyer_amount
    let seller_amount = net.checked_sub(buyer_amount).ok_or(ContractError::Overflow)?;
    
    Ok((buyer_amount, seller_amount))
}

/// Validate that a partial release is feasible for a given trade.
/// 
/// Checks if the trade exists, is in disputed status, and the split
/// parameters are valid.
/// 
/// # Arguments
/// * `env` - The Soroban environment
/// * `trade_id` - The trade ID to validate
/// * `buyer_bps` - The buyer's share in basis points (0-10000)
/// 
/// # Returns
/// * `Ok(())` - If validation passes
/// * `Err(ContractError)` - If validation fails
pub fn validate_partial_release(
    env: &Env,
    trade_id: u64,
    buyer_bps: u32,
) -> Result<(), ContractError> {
    // Validate buyer_bps range
    if buyer_bps > 10000 {
        return Err(ContractError::InvalidSplitBps);
    }
    
    // Get trade and validate it exists
    let trade = get_trade(env, trade_id)?;
    
    // Validate trade is in disputed status
    if trade.status != crate::types::TradeStatus::Disputed {
        return Err(ContractError::InvalidStatus);
    }
    
    // Calculate amounts to ensure no overflow
    let _ = calculate_partial_amounts(env, trade_id, buyer_bps)?;
    
    Ok(())
}

/// Get the fee amount for a partial release.
/// 
/// The fee is calculated on the full trade amount before distribution.
/// 
/// # Arguments
/// * `env` - The Soroban environment
/// * `trade_id` - The trade ID
/// 
/// # Returns
/// * `Ok(fee_amount)` - The fee amount
/// * `Err(ContractError)` - If trade not found
pub fn get_partial_release_fee(
    env: &Env,
    trade_id: u64,
) -> Result<u64, ContractError> {
    let trade = get_trade(env, trade_id)?;
    Ok(trade.fee)
}
