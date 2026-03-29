use soroban_sdk::{token, Address, Env};

use crate::errors::ContractError;
use crate::events;
use crate::storage::{get_subscription, get_usdc_token, remove_subscription, save_subscription};
use crate::types::{
    Subscription, SubscriptionTier, SUBSCRIPTION_DURATION_LEDGERS, SUB_DISCOUNT_BASIC_BPS,
    SUB_DISCOUNT_ENTERPRISE_BPS, SUB_DISCOUNT_PRO_BPS, SUB_PRICE_BASIC, SUB_PRICE_ENTERPRISE,
    SUB_PRICE_PRO,
};

fn tier_price(tier: &SubscriptionTier) -> u64 {
    match tier {
        SubscriptionTier::Basic => SUB_PRICE_BASIC,
        SubscriptionTier::Pro => SUB_PRICE_PRO,
        SubscriptionTier::Enterprise => SUB_PRICE_ENTERPRISE,
    }
}

/// Returns the fee discount in bps for an active subscription, or 0 if none/expired.
pub fn subscription_discount_bps(env: &Env, user: &Address) -> u32 {
    match get_subscription(env, user) {
        Some(sub) if env.ledger().sequence() <= sub.expires_at => match sub.tier {
            SubscriptionTier::Basic => SUB_DISCOUNT_BASIC_BPS,
            SubscriptionTier::Pro => SUB_DISCOUNT_PRO_BPS,
            SubscriptionTier::Enterprise => SUB_DISCOUNT_ENTERPRISE_BPS,
        },
        _ => 0,
    }
}

/// Purchase a new subscription. Fails if one is already active.
pub fn subscribe(
    env: &Env,
    subscriber: &Address,
    tier: SubscriptionTier,
    admin: &Address,
) -> Result<(), ContractError> {
    if let Some(existing) = get_subscription(env, subscriber) {
        if env.ledger().sequence() <= existing.expires_at {
            return Err(ContractError::SubscriptionAlreadyActive);
        }
    }

    let price = tier_price(&tier);
    let token = get_usdc_token(env)?;
    let token_client = token::Client::new(env, &token);
    token_client.transfer(subscriber, admin, &(price as i128));

    let now = env.ledger().sequence();
    let expires_at = now
        .checked_add(SUBSCRIPTION_DURATION_LEDGERS)
        .ok_or(ContractError::Overflow)?;

    let sub = Subscription {
        subscriber: subscriber.clone(),
        tier: tier.clone(),
        expires_at,
        renewed_at: now,
    };
    save_subscription(env, subscriber, &sub);
    events::emit_subscribed(env, subscriber.clone(), tier, expires_at);
    Ok(())
}

/// Renew an existing subscription (active or expired). Extends from current expiry if active,
/// or from now if expired.
pub fn renew(env: &Env, subscriber: &Address, admin: &Address) -> Result<(), ContractError> {
    let sub = get_subscription(env, subscriber).ok_or(ContractError::SubscriptionNotFound)?;

    let price = tier_price(&sub.tier);
    let token = get_usdc_token(env)?;
    let token_client = token::Client::new(env, &token);
    token_client.transfer(subscriber, admin, &(price as i128));

    let now = env.ledger().sequence();
    let base = if now <= sub.expires_at { sub.expires_at } else { now };
    let new_expires = base
        .checked_add(SUBSCRIPTION_DURATION_LEDGERS)
        .ok_or(ContractError::Overflow)?;

    let updated = Subscription {
        subscriber: subscriber.clone(),
        tier: sub.tier.clone(),
        expires_at: new_expires,
        renewed_at: now,
    };
    save_subscription(env, subscriber, &updated);
    events::emit_subscription_renewed(env, subscriber.clone(), sub.tier, new_expires);
    Ok(())
}

/// Cancel a subscription immediately (no refund).
pub fn cancel(env: &Env, subscriber: &Address) -> Result<(), ContractError> {
    if get_subscription(env, subscriber).is_none() {
        return Err(ContractError::SubscriptionNotFound);
    }
    remove_subscription(env, subscriber);
    events::emit_subscription_cancelled(env, subscriber.clone());
    Ok(())
}

/// Get subscription details for a user.
pub fn get(env: &Env, subscriber: &Address) -> Option<Subscription> {
    get_subscription(env, subscriber)
}

/// Check if a user has an active subscription.
///
/// Returns true if the user has a subscription that hasn't expired.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `subscriber` - The user's address
///
/// # Returns
/// * `bool` - True if subscription is active, false otherwise
pub fn is_subscription_active(env: &Env, subscriber: &Address) -> bool {
    match get_subscription(env, subscriber) {
        Some(sub) => env.ledger().sequence() <= sub.expires_at,
        None => false,
    }
}

/// Get the subscription tier for a user.
///
/// Returns the tier if the user has an active subscription, None otherwise.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `subscriber` - The user's address
///
/// # Returns
/// * `Option<SubscriptionTier>` - The tier if active, None otherwise
pub fn get_subscription_tier(env: &Env, subscriber: &Address) -> Option<SubscriptionTier> {
    match get_subscription(env, subscriber) {
        Some(sub) if env.ledger().sequence() <= sub.expires_at => Some(sub.tier),
        _ => None,
    }
}

/// Get the fee discount for a user's subscription.
///
/// Returns the discount in basis points (0-10000) if the user has an active subscription.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `subscriber` - The user's address
///
/// # Returns
/// * `u32` - Discount in basis points (0 if no active subscription)
pub fn get_subscription_discount(env: &Env, subscriber: &Address) -> u32 {
    match get_subscription(env, subscriber) {
        Some(sub) if env.ledger().sequence() <= sub.expires_at => match sub.tier {
            SubscriptionTier::Basic => SUB_DISCOUNT_BASIC_BPS,
            SubscriptionTier::Pro => SUB_DISCOUNT_PRO_BPS,
            SubscriptionTier::Enterprise => SUB_DISCOUNT_ENTERPRISE_BPS,
        },
        _ => 0,
    }
}

/// Get the subscription price for a tier.
///
/// Returns the price in USDC for the given tier.
///
/// # Arguments
/// * `tier` - The subscription tier
///
/// # Returns
/// * `u64` - Price in USDC
pub fn get_subscription_price(tier: &SubscriptionTier) -> u64 {
    tier_price(tier)
}

/// Check if a subscription is expired.
///
/// Returns true if the subscription exists but has expired.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `subscriber` - The user's address
///
/// # Returns
/// * `bool` - True if subscription is expired, false otherwise
pub fn is_subscription_expired(env: &Env, subscriber: &Address) -> bool {
    match get_subscription(env, subscriber) {
        Some(sub) => env.ledger().sequence() > sub.expires_at,
        None => false,
    }
}
