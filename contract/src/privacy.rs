use soroban_sdk::{Address, Env, String};

use crate::errors::ContractError;
use crate::events;
use crate::storage::{
    get_disclosure_grant, get_trade, get_trade_privacy, remove_disclosure_grant,
    save_disclosure_grant, save_trade_privacy,
};
use crate::types::{DisclosureGrant, TradePrivacy, PRIVACY_DATA_PTR_MAX_LEN};

fn require_trade_party(trade_id: u64, caller: &Address, env: &Env) -> Result<(), ContractError> {
    let trade = get_trade(env, trade_id)?;
    if *caller != trade.seller && *caller != trade.buyer {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

/// Set or update privacy settings for a trade (seller or buyer only).
pub fn set_trade_privacy(
    env: &Env,
    caller: &Address,
    trade_id: u64,
    data_hash: String,
    encrypted_ptr: Option<String>,
    private_arbitration: bool,
) -> Result<(), ContractError> {
    require_trade_party(trade_id, caller, env)?;
    if let Some(ref ptr) = encrypted_ptr {
        if ptr.len() > PRIVACY_DATA_PTR_MAX_LEN {
            return Err(ContractError::PrivacyDataTooLong);
        }
    }
    let privacy = TradePrivacy { data_hash, encrypted_ptr, private_arbitration };
    save_trade_privacy(env, trade_id, &privacy);
    events::emit_privacy_set(env, trade_id);
    Ok(())
}

/// Grant selective disclosure to a third party (seller or buyer only).
pub fn grant_disclosure(
    env: &Env,
    caller: &Address,
    trade_id: u64,
    grantee: Address,
    encrypted_key: String,
) -> Result<(), ContractError> {
    require_trade_party(trade_id, caller, env)?;
    if encrypted_key.len() > PRIVACY_DATA_PTR_MAX_LEN {
        return Err(ContractError::PrivacyDataTooLong);
    }
    let grant = DisclosureGrant {
        trade_id,
        grantee: grantee.clone(),
        encrypted_key,
    };
    save_disclosure_grant(env, trade_id, &grantee, &grant);
    events::emit_disclosure_granted(env, trade_id, grantee);
    Ok(())
}

/// Revoke a previously granted disclosure (seller or buyer only).
pub fn revoke_disclosure(
    env: &Env,
    caller: &Address,
    trade_id: u64,
    grantee: Address,
) -> Result<(), ContractError> {
    require_trade_party(trade_id, caller, env)?;
    remove_disclosure_grant(env, trade_id, &grantee);
    events::emit_disclosure_revoked(env, trade_id, grantee);
    Ok(())
}

/// Query privacy settings. Arbitrator identity hidden if private_arbitration is set.
pub fn get_privacy(env: &Env, trade_id: u64) -> Option<TradePrivacy> {
    get_trade_privacy(env, trade_id)
}

/// Query a disclosure grant for a specific grantee (grantee can call this to retrieve their key).
pub fn get_grant(
    env: &Env,
    trade_id: u64,
    grantee: &Address,
) -> Result<DisclosureGrant, ContractError> {
    get_disclosure_grant(env, trade_id, grantee)
        .ok_or(ContractError::DisclosureGrantNotFound)
}

/// Check whether a trade has privacy settings configured.
pub fn has_privacy(env: &Env, trade_id: u64) -> bool {
    get_trade_privacy(env, trade_id).is_some()
}

/// Check whether a specific grantee has an active disclosure grant for a trade.
pub fn has_disclosure_grant(env: &Env, trade_id: u64, grantee: &Address) -> bool {
    get_disclosure_grant(env, trade_id, grantee).is_some()
}

/// Verify that a data hash matches the stored privacy commitment.
/// Returns `Ok(true)` if the hash matches, `Ok(false)` if it doesn't,
/// or `Err(DisclosureGrantNotFound)` if no privacy record exists.
pub fn verify_data_hash(
    env: &Env,
    trade_id: u64,
    claimed_hash: &String,
) -> Result<bool, ContractError> {
    let privacy = get_trade_privacy(env, trade_id)
        .ok_or(ContractError::DisclosureGrantNotFound)?;
    Ok(privacy.data_hash == *claimed_hash)
}

/// Privacy compliance check: ensure a trade has a privacy record before
/// allowing sensitive operations (e.g. private arbitration).
/// Returns `Err(DisclosureUnauthorized)` if private_arbitration is requested
/// but no privacy record has been set.
pub fn require_privacy_compliance(
    env: &Env,
    trade_id: u64,
    requires_private_arbitration: bool,
) -> Result<(), ContractError> {
    if !requires_private_arbitration {
        return Ok(());
    }
    let privacy = get_trade_privacy(env, trade_id)
        .ok_or(ContractError::DisclosureUnauthorized)?;
    if !privacy.private_arbitration {
        return Err(ContractError::DisclosureUnauthorized);
    }
    Ok(())
}
