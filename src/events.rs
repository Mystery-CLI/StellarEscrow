use soroban_sdk::{symbol_short, Address, Env};

use crate::types::DisputeResolution;

pub fn emit_trade_created(env: &Env, trade_id: u64, seller: Address, buyer: Address, amount: u64) {
    env.events().publish((symbol_short!("created"),), (trade_id, seller, buyer, amount));
}

pub fn emit_trade_funded(env: &Env, trade_id: u64) {
    env.events().publish((symbol_short!("funded"),), trade_id);
}

pub fn emit_trade_completed(env: &Env, trade_id: u64) {
    env.events().publish((symbol_short!("complete"),), trade_id);
}

pub fn emit_trade_confirmed(env: &Env, trade_id: u64, payout: u64, fee: u64) {
    env.events().publish((symbol_short!("confirm"),), (trade_id, payout, fee));
}

pub fn emit_dispute_raised(env: &Env, trade_id: u64, raised_by: Address) {
    env.events().publish((symbol_short!("dispute"),), (trade_id, raised_by));
}

pub fn emit_dispute_resolved(env: &Env, trade_id: u64, resolution: DisputeResolution, recipient: Address) {
    env.events().publish((symbol_short!("resolved"),), (trade_id, resolution, recipient));
}

pub fn emit_trade_cancelled(env: &Env, trade_id: u64) {
    env.events().publish((symbol_short!("cancel"),), trade_id);
}

pub fn emit_arbitrator_registered(env: &Env, arbitrator: Address) {
    env.events().publish((symbol_short!("arb_reg"),), arbitrator);
}

pub fn emit_arbitrator_removed(env: &Env, arbitrator: Address) {
    env.events().publish((symbol_short!("arb_rem"),), arbitrator);
}

pub fn emit_metadata_updated(env: &Env, trade_id: u64) {
    env.events().publish((symbol_short!("meta_upd"),), trade_id);
}

pub fn emit_fee_updated(env: &Env, fee_bps: u32) {
    env.events().publish((symbol_short!("fee_upd"),), fee_bps);
}

pub fn emit_fees_withdrawn(env: &Env, amount: u64, to: Address) {
    env.events().publish((symbol_short!("fees_out"),), (amount, to));
}

pub fn emit_batch_trades_created(env: &Env, count: u32, total_amount: u64) {
    env.events().publish((symbol_short!("batch_cr"),), (count, total_amount));
}

pub fn emit_batch_trades_funded(env: &Env, count: u32, total_amount: u64) {
    env.events().publish((symbol_short!("batch_fd"),), (count, total_amount));
}

pub fn emit_batch_trades_confirmed(env: &Env, count: u32, total_payout: u64, total_fees: u64) {
    env.events().publish((symbol_short!("batch_cn"),), (count, total_payout, total_fees));
}

pub fn emit_paused(env: &Env, admin: Address) {
    env.events().publish((symbol_short!("paused"),), admin);
}

pub fn emit_unpaused(env: &Env, admin: Address) {
    env.events().publish((symbol_short!("unpaused"),), admin);
}

pub fn emit_emergency_withdraw(env: &Env, to: Address, amount: u64) {
    env.events().publish((symbol_short!("emrg_wd"),), (to, amount));
}

pub fn emit_tier_upgraded(env: &Env, user: Address, new_tier: crate::types::UserTier) {
    env.events().publish((symbol_short!("tier_up"),), (user, new_tier));
}

pub fn emit_tier_downgraded(env: &Env, user: Address, new_tier: crate::types::UserTier) {
    env.events().publish((symbol_short!("tier_dn"),), (user, new_tier));
}

pub fn emit_tier_config_updated(env: &Env) {
    env.events().publish((symbol_short!("tier_cfg"),), ());
}

pub fn emit_custom_fee_set(env: &Env, user: Address, fee_bps: u32) {
    env.events().publish((symbol_short!("cust_fee"),), (user, fee_bps));
}

pub fn emit_template_created(env: &Env, template_id: u64, owner: Address) {
    env.events().publish((symbol_short!("tmpl_cr"),), (template_id, owner));
}

pub fn emit_template_updated(env: &Env, template_id: u64, version: u32) {
    env.events().publish((symbol_short!("tmpl_up"),), (template_id, version));
}

pub fn emit_template_deactivated(env: &Env, template_id: u64) {
    env.events().publish((symbol_short!("tmpl_off"),), template_id);
}

pub fn emit_trade_from_template(env: &Env, trade_id: u64, template_id: u64, version: u32) {
    env.events().publish((symbol_short!("tmpl_tr"),), (trade_id, template_id, version));
}

pub fn emit_preset_saved(env: &Env, owner: Address, preset_id: u64) {
    env.events().publish((symbol_short!("pst_save"),), (owner, preset_id));
}

pub fn emit_preset_deleted(env: &Env, owner: Address, preset_id: u64) {
    env.events().publish((symbol_short!("pst_del"),), (owner, preset_id));
pub fn emit_analytics_exported(env: &Env, export_type: u32) {
    env.events().publish((symbol_short!("anlt_exp"),), export_type);
}

pub fn emit_onboarding_started(env: &Env, user: Address) {
    env.events()
        .publish((symbol_short!("ob_start"),), user);
}

pub fn emit_onboarding_step_done(env: &Env, user: Address, step_index: u32) {
    env.events()
        .publish((symbol_short!("ob_step"),), (user, step_index));
}

pub fn emit_onboarding_step_skipped(env: &Env, user: Address, step_index: u32) {
    env.events()
        .publish((symbol_short!("ob_skip"),), (user, step_index));
}

pub fn emit_onboarding_completed(env: &Env, user: Address) {
    env.events()
        .publish((symbol_short!("ob_done"),), user);
}

pub fn emit_onboarding_exited(env: &Env, user: Address) {
    env.events()
        .publish((symbol_short!("ob_exit"),), user);
pub fn emit_avatar_updated(env: &Env, address: Address) {
    env.events()
        .publish((symbol_short!("avtr_up"),), address);
}

pub fn emit_security_updated(env: &Env, address: Address, two_fa_enabled: bool) {
    env.events()
        .publish((symbol_short!("sec_up"),), (address, two_fa_enabled));
}
