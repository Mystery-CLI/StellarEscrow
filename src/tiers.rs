use soroban_sdk::{Address, Env};

use crate::errors::ContractError;
use crate::events;
use crate::storage::{get_tier_config, get_user_tier, save_user_tier};
use crate::types::{TierConfig, UserTier, UserTierInfo, TIER_GOLD_THRESHOLD, TIER_SILVER_THRESHOLD};

/// Resolve the effective fee bps for a user, falling back to the platform base fee.
///
/// Priority: Custom > tier config > base_fee_bps
pub fn effective_fee_bps(env: &Env, user: &Address, base_fee_bps: u32) -> u32 {
    let info = match get_user_tier(env, user) {
        Some(i) => i,
        None => return base_fee_bps,
    };

    // Custom tier overrides everything
    if let UserTier::Custom = info.tier {
        if let Some(custom) = info.custom_fee_bps {
            return custom;
        }
    }

    // Use tier config if set, otherwise fall back to base
    match get_tier_config(env) {
        Some(cfg) => match info.tier {
            UserTier::Bronze => cfg.bronze_fee_bps,
            UserTier::Silver => cfg.silver_fee_bps,
            UserTier::Gold => cfg.gold_fee_bps,
            UserTier::Custom => base_fee_bps, // custom_fee_bps was None, fall back
        },
        None => base_fee_bps,
    }
}

/// Derive the volume-based tier for a given cumulative volume.
fn volume_tier(total_volume: u64) -> UserTier {
    if total_volume >= TIER_GOLD_THRESHOLD {
        UserTier::Gold
    } else if total_volume >= TIER_SILVER_THRESHOLD {
        UserTier::Silver
    } else {
        UserTier::Bronze
    }
}

/// Record completed trade volume for a user and auto-upgrade/downgrade their tier.
/// Called after a trade is confirmed (funds released).
pub fn record_volume(env: &Env, user: &Address, amount: u64) -> Result<(), ContractError> {
    let mut info = get_user_tier(env, user).unwrap_or(UserTierInfo {
        tier: UserTier::Bronze,
        total_volume: 0,
        custom_fee_bps: None,
    });

    // Don't touch custom-tier users — their fee is manually managed
    if let UserTier::Custom = info.tier {
        info.total_volume = info
            .total_volume
            .checked_add(amount)
            .ok_or(ContractError::Overflow)?;
        save_user_tier(env, user, &info);
        return Ok(());
    }

    let old_tier = info.tier.clone();
    info.total_volume = info
        .total_volume
        .checked_add(amount)
        .ok_or(ContractError::Overflow)?;
    let new_tier = volume_tier(info.total_volume);

    let tier_changed = old_tier != new_tier;
    let upgraded = matches!(
        (&old_tier, &new_tier),
        (UserTier::Bronze, UserTier::Silver)
            | (UserTier::Bronze, UserTier::Gold)
            | (UserTier::Silver, UserTier::Gold)
    );

    info.tier = new_tier.clone();
    save_user_tier(env, user, &info);

    if tier_changed {
        if upgraded {
            events::emit_tier_upgraded(env, user.clone(), new_tier);
        } else {
            events::emit_tier_downgraded(env, user.clone(), new_tier);
        }
    }

    Ok(())
}

/// Admin: set or update the tier fee configuration.
pub fn set_tier_config(env: &Env, config: &TierConfig) -> Result<(), ContractError> {
    if config.bronze_fee_bps > 10000
        || config.silver_fee_bps > 10000
        || config.gold_fee_bps > 10000
    {
        return Err(ContractError::InvalidTierConfig);
    }
    // Enforce sensible ordering: bronze >= silver >= gold
    if config.silver_fee_bps > config.bronze_fee_bps
        || config.gold_fee_bps > config.silver_fee_bps
    {
        return Err(ContractError::InvalidTierConfig);
    }
    crate::storage::save_tier_config(env, config);
    events::emit_tier_config_updated(env);
    Ok(())
}

/// Admin: assign a custom fee rate to a specific user.
pub fn set_custom_fee(env: &Env, user: &Address, fee_bps: u32) -> Result<(), ContractError> {
    if fee_bps > 10000 {
        return Err(ContractError::InvalidFeeBps);
    }
    let mut info = get_user_tier(env, user).unwrap_or(UserTierInfo {
        tier: UserTier::Custom,
        total_volume: 0,
        custom_fee_bps: None,
    });
    info.tier = UserTier::Custom;
    info.custom_fee_bps = Some(fee_bps);
    save_user_tier(env, user, &info);
    events::emit_custom_fee_set(env, user.clone(), fee_bps);
    Ok(())
}

/// Admin: remove a custom fee, reverting the user to volume-based tier.
pub fn remove_custom_fee(env: &Env, user: &Address) {
    let mut info = get_user_tier(env, user).unwrap_or(UserTierInfo {
        tier: UserTier::Bronze,
        total_volume: 0,
        custom_fee_bps: None,
    });
    info.custom_fee_bps = None;
    info.tier = volume_tier(info.total_volume);
    save_user_tier(env, user, &info);
}
