use soroban_sdk::{Env, Address, Bytes, String};

use crate::errors::ContractError;
use crate::storage::{
    get_analytics, get_preference, get_user, has_user, save_analytics, save_preference, save_user,
};
use crate::types::{UserAnalytics, UserPreference, UserProfile, VerificationStatus};
use crate::events;

/// Register a new user profile.
/// `username_hash` and `contact_hash` are SHA-256 hashes computed off-chain.
pub fn register_user(
    env: &Env,
    address: Address,
    username_hash: Bytes,
    contact_hash: Bytes,
) -> Result<(), ContractError> {
    if has_user(env, &address) {
        return Err(ContractError::AlreadyInitialized); // reuse: user already exists
    }
    address.require_auth();
    let now = env.ledger().sequence();
    let profile = UserProfile {
        address: address.clone(),
        username_hash,
        contact_hash,
        avatar_hash: None,
        verification: VerificationStatus::Unverified,
        two_fa_enabled: false,
        session_timeout_secs: 0,
        registered_at: now,
        updated_at: now,
    };
    save_user(env, &profile);
    Ok(())
}

/// Update an existing user's profile hashes.
pub fn update_profile(
    env: &Env,
    address: Address,
    username_hash: Bytes,
    contact_hash: Bytes,
) -> Result<(), ContractError> {
    let mut profile = get_user(env, &address).ok_or(ContractError::TradeNotFound)?;
    address.require_auth();
    profile.username_hash = username_hash;
    profile.contact_hash = contact_hash;
    profile.updated_at = env.ledger().sequence();
    save_user(env, &profile);
    Ok(())
}

/// Get a user profile.
pub fn get_profile(env: &Env, address: &Address) -> Result<UserProfile, ContractError> {
    get_user(env, address).ok_or(ContractError::TradeNotFound)
}

/// Set or update a user preference key/value pair.
pub fn set_preference(
    env: &Env,
    address: Address,
    key: String,
    value: String,
) -> Result<(), ContractError> {
    if !has_user(env, &address) {
        return Err(ContractError::TradeNotFound);
    }
    address.require_auth();
    let pref = UserPreference { key, value };
    save_preference(env, &address, &pref);
    Ok(())
}

/// Get a user preference by key.
pub fn get_pref(
    env: &Env,
    address: &Address,
    key: &String,
) -> Result<UserPreference, ContractError> {
    get_preference(env, address, key).ok_or(ContractError::TradeNotFound)
}

/// Set verification status for a user (admin only — caller must verify admin auth before calling).
pub fn set_verification(
    env: &Env,
    address: &Address,
    status: VerificationStatus,
) -> Result<(), ContractError> {
    let mut profile = get_user(env, address).ok_or(ContractError::TradeNotFound)?;
    profile.verification = status;
    profile.updated_at = env.ledger().sequence();
    save_user(env, &profile);
    Ok(())
}

/// Get analytics for a user.
pub fn get_user_analytics(env: &Env, address: &Address) -> UserAnalytics {
    get_analytics(env, address)
}

/// Update analytics when a trade is created.
pub fn record_trade_created(env: &Env, seller: &Address, buyer: &Address, amount: u64) {
    for (address, is_seller) in [(seller, true), (buyer, false)] {
        let mut stats = get_analytics(env, address);
        stats.total_trades += 1;
        stats.total_volume = stats.total_volume.saturating_add(amount);
        if is_seller {
            stats.trades_as_seller += 1;
        } else {
            stats.trades_as_buyer += 1;
        }
        save_analytics(env, &stats);
    }
}

/// Update analytics when a trade is completed.
pub fn record_trade_completed(env: &Env, seller: &Address, buyer: &Address) {
    for address in [seller, buyer] {
        let mut stats = get_analytics(env, address);
        stats.completed_trades += 1;
        save_analytics(env, &stats);
    }
}

/// Update analytics when a dispute is raised.
pub fn record_trade_disputed(env: &Env, seller: &Address, buyer: &Address) {
    for address in [seller, buyer] {
        let mut stats = get_analytics(env, address);
        stats.disputed_trades += 1;
        save_analytics(env, &stats);
    }
}

/// Update analytics when a trade is cancelled.
pub fn record_trade_cancelled(env: &Env, seller: &Address) {
    let mut stats = get_analytics(env, seller);
    stats.cancelled_trades += 1;
    save_analytics(env, &stats);
}

/// Update the avatar hash for a user (SHA-256 of the off-chain image).
pub fn update_avatar(
    env: &Env,
    address: Address,
    avatar_hash: Option<Bytes>,
) -> Result<(), ContractError> {
    let mut profile = get_user(env, &address).ok_or(ContractError::TradeNotFound)?;
    address.require_auth();
    profile.avatar_hash = avatar_hash;
    profile.updated_at = env.ledger().sequence();
    save_user(env, &profile);
    events::emit_avatar_updated(env, address);
    Ok(())
}

/// Update security settings (2FA flag and session timeout) for a user.
pub fn update_security_settings(
    env: &Env,
    address: Address,
    two_fa_enabled: bool,
    session_timeout_secs: u32,
) -> Result<(), ContractError> {
    let mut profile = get_user(env, &address).ok_or(ContractError::TradeNotFound)?;
    address.require_auth();
    profile.two_fa_enabled = two_fa_enabled;
    profile.session_timeout_secs = session_timeout_secs;
    profile.updated_at = env.ledger().sequence();
    save_user(env, &profile);
    events::emit_security_updated(env, address, two_fa_enabled);
    Ok(())
}
