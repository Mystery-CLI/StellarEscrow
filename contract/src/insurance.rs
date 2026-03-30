use soroban_sdk::{token, Address, Env};
use crate::storage::{
    get_insurance_policy, has_insurance_provider, save_insurance_policy,
    get_trade,
};
use crate::types::{TradeStatus, InsurancePolicy};
use crate::errors::ContractError;
use crate::events;

/// Calculate insurance premium based on trade amount (e.g., 2% flat for this implementation)
/// Supports multiple providers by allowing them to define premiums (simulated here)
pub fn calculate_premium(_env: &Env, amount: u64, _provider: &Address) -> u64 {
    // 2% premium (200 bps)
    amount * 200 / 10000
}

/// Attach an insurance policy to a trade
pub fn purchase_insurance(
    env: &Env,
    trade_id: u64,
    buyer: Address,
    provider: Address,
) -> Result<(), ContractError> {
    if !has_insurance_provider(env, &provider) {
        return Err(ContractError::InsuranceProviderNotRegistered);
    }

    let mut trade = get_trade(env, trade_id)?;
    if trade.status != TradeStatus::Created {
        return Err(ContractError::InvalidStatus);
    }
    if trade.buyer != buyer {
        return Err(ContractError::Unauthorized);
    }
    buyer.require_auth();

    if get_insurance_policy(env, trade_id).is_some() {
        return Err(ContractError::InvalidStatus); // Already insured
    }

    let premium = calculate_premium(env, trade.amount, &provider);
    if premium > (trade.amount * crate::types::MAX_INSURANCE_PREMIUM_BPS as u64 / 10000) {
        return Err(ContractError::InsurancePremiumTooHigh);
    }

    let token_client = token::Client::new(env, &trade.currency);
    token_client.transfer(&buyer, &env.current_contract_address(), &(premium as i128));

    let coverage = trade.amount; // 100% coverage
    let policy = InsurancePolicy {
        provider: provider.clone(),
        premium,
        coverage,
        claimed: false,
    };

    save_insurance_policy(env, trade_id, &policy);
    events::emit_insurance_purchased(env, trade_id, provider, premium, coverage);

    Ok(())
}

/// Process an insurance claim after a dispute resolution
pub fn claim_insurance(
    env: &Env,
    trade_id: u64,
    recipient: Address,
) -> Result<(), ContractError> {
    let mut policy = get_insurance_policy(env, trade_id).ok_or(ContractError::TradeNotInsured)?;
    if policy.claimed {
        return Err(ContractError::InsuranceAlreadyClaimed);
    }

    let trade = get_trade(env, trade_id)?;
    if trade.status != TradeStatus::Disputed {
        return Err(ContractError::InvalidStatus);
    }

    // Only allow claims if the recipient was potentially wronged 
    // (e.g., seller sends junk, buyer loses money)
    // For simplicity, we assume the claim is valid if the trade was disputed 
    // and the insurance provider authorizes it or it's triggered by specific resolution
    recipient.require_auth();

    let payout = policy.coverage;
    let token_client = token::Client::new(env, &trade.currency);
    
    // Transfer from provider to recipient
    token_client.transfer(&policy.provider, &recipient, &(payout as i128));

    policy.claimed = true;
    save_insurance_policy(env, trade_id, &policy);

    events::emit_insurance_claimed(env, trade_id, payout, recipient);

    Ok(())
}
