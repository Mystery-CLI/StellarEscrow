//! Emergency Pause Helper Functions
//! 
//! This module provides helper functions for emergency pause functionality,
//! including pause status checks and emergency withdrawal calculations.

use soroban_sdk::{token, Address, Env};
use crate::errors::ContractError;
use crate::storage::{get_usdc_token, is_paused};

/// Check if the contract is currently in paused state.
/// 
/// Returns true if the contract is paused, false otherwise.
/// 
/// # Arguments
/// * `env` - The Soroban environment
/// 
/// # Returns
/// * `bool` - True if paused, false otherwise
/// 
/// # Example
/// ```
/// if is_contract_paused(&env) {
///     // Contract is paused, only emergency operations allowed
/// }
/// ```
pub fn is_contract_paused(env: &Env) -> bool {
    is_paused(env)
}

/// Get the current contract token balance.
/// 
/// Useful for emergency withdrawal planning.
/// 
/// # Arguments
/// * `env` - The Soroban environment
/// 
/// # Returns
/// * `Ok(balance)` - The current token balance
/// * `Err(ContractError)` - If token client fails
pub fn get_contract_balance(env: &Env) -> Result<u64, ContractError> {
    let token = get_usdc_token(env)?;
    let token_client = token::Client::new(env, &token);
    let contract_address = env.current_contract_address();
    let balance = token_client.balance(&contract_address);
    Ok(balance as u64)
}

/// Validate that an emergency withdrawal is possible.
/// 
/// Checks if the contract has sufficient balance for the withdrawal.
/// 
/// # Arguments
/// * `env` - The Soroban environment
/// * `amount` - The amount to withdraw
/// 
/// # Returns
/// * `Ok(())` - If withdrawal is possible
/// * `Err(ContractError)` - If insufficient balance
pub fn validate_emergency_withdrawal(
    env: &Env,
    amount: u64,
) -> Result<(), ContractError> {
    let balance = get_contract_balance(env)?;
    if amount > balance {
        return Err(ContractError::Overflow);
    }
    Ok(())
}

/// Get pause status with additional context.
/// 
/// Returns a tuple of (is_paused, can_perform_operations).
/// 
/// # Arguments
/// * `env` - The Soroban environment
/// 
/// # Returns
/// * `(bool, bool)` - (is_paused, can_perform_operations)
pub fn get_pause_status(env: &Env) -> (bool, bool) {
    let paused = is_paused(env);
    let can_operate = !paused;
    (paused, can_operate)
}

/// Check if a specific operation is allowed in the current state.
/// 
/// Emergency operations (like emergency_withdraw) are always allowed.
/// Regular operations are only allowed when not paused.
/// 
/// # Arguments
/// * `env` - The Soroban environment
/// * `is_emergency_operation` - Whether this is an emergency operation
/// 
/// # Returns
/// * `Ok(())` - If operation is allowed
/// * `Err(ContractError::ContractPaused)` - If operation is not allowed
pub fn check_operation_allowed(
    env: &Env,
    is_emergency_operation: bool,
) -> Result<(), ContractError> {
    if is_emergency_operation {
        // Emergency operations always allowed
        return Ok(());
    }
    
    // Regular operations require contract to not be paused
    if is_paused(env) {
        return Err(ContractError::ContractPaused);
    }
    
    Ok(())
}
