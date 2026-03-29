use soroban_sdk::{Address, Env};

use crate::errors::ContractError;
use crate::events;
use crate::storage::{get_tier_config, get_user_tier, save_user_tier};
use crate::types::{TierConfig, UserTier, UserTierInfo, TIER_GOLD_THRESHOLD, TIER_SILVER_THRESHOLD};

pub fn effective_fee_bps(env: &Env, user: &Address, base_fee_bps: u32) -> u32 {
    let info = match get_user_tier(env, user) {
        Some(i) => i,
        None => return base_fee_bps,
    };
    if let UserTier::Custom = info.tier {
        if let Some(custom) = info.custom_fee_bps {
            return custom;
        }
    }
    match get_tier_config(env) {
        Some(cfg) => match info.tier {
            UserTier::Bronze => cfg.bronze_fee_bps,
            UserTier::Silver => cfg.silver_fee_bps,
            UserTier::Gold => cfg.gold_fee_bps,
            UserTier::Custom => base_fee_bps,
        },
        None => base_fee_bps,
    }
}

fn volume_tier(total_volume: u64) -> UserTier {
    if total_volume >= TIER_GOLD_THRESHOLD {
        UserTier::Gold
    } else if total_volume >= TIER_SILVER_THRESHOLD {
        UserTier::Silver
    } else {
        UserTier::Bronze
    }
}

pub fn record_volume(env: &Env, user: &Address, amount: u64) -> Result<(), ContractError> {
    let mut info = get_user_tier(env, user).unwrap_or(UserTierInfo {
        tier: UserTier::Bronze,
        total_volume: 0,
        custom_fee_bps: None,
    });
    if let UserTier::Custom = info.tier {
        info.total_volume = info.total_volume.checked_add(amount).ok_or(ContractError::Overflow)?;
        save_user_tier(env, user, &info);
        return Ok(());
    }
    info.total_volume = info.total_volume.checked_add(amount).ok_or(ContractError::Overflow)?;
    let new_tier = volume_tier(info.total_volume);
    if new_tier != info.tier {
        let upgraded = matches!(
            (&info.tier, &new_tier),
            (UserTier::Bronze, UserTier::Silver)
                | (UserTier::Bronze, UserTier::Gold)
                | (UserTier::Silver, UserTier::Gold)
        );
        info.tier = new_tier.clone();
        save_user_tier(env, user, &info);
        if upgraded {
            events::emit_tier_upgraded(env, user.clone(), new_tier);
        } else {
            events::emit_tier_downgraded(env, user.clone(), new_tier);
        }
    } else {
        save_user_tier(env, user, &info);
    }
    Ok(())
}

pub fn set_tier_config(env: &Env, config: &TierConfig) -> Result<(), ContractError> {
    if config.bronze_fee_bps > 10000
        || config.silver_fee_bps > 10000
        || config.gold_fee_bps > 10000
    {
        return Err(ContractError::InvalidTierConfig);
    }
    if config.silver_fee_bps > config.bronze_fee_bps
        || config.gold_fee_bps > config.silver_fee_bps
    {
        return Err(ContractError::InvalidTierConfig);
    }
    crate::storage::save_tier_config(env, config);
    events::emit_tier_config_updated(env);
    Ok(())
}

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
