use soroban_sdk::{symbol_short, Address, Env};

use crate::types::DisputeResolution;

pub fn emit_trade_created(env: &Env, trade_id: u64, seller: Address, buyer: Address, amount: u64) {
    env.events().publish(
        (symbol_short!("created"),),
        (trade_id, seller, buyer, amount),
    );
}

pub fn emit_trade_funded(env: &Env, trade_id: u64) {
    env.events().publish((symbol_short!("funded"),), trade_id);
}

pub fn emit_trade_completed(env: &Env, trade_id: u64) {
    env.events()
        .publish((symbol_short!("complete"),), trade_id);
}

pub fn emit_trade_confirmed(env: &Env, trade_id: u64, payout: u64, fee: u64) {
    env.events()
        .publish((symbol_short!("confirm"),), (trade_id, payout, fee));
}

pub fn emit_dispute_raised(env: &Env, trade_id: u64, raised_by: Address) {
    env.events()
        .publish((symbol_short!("dispute"),), (trade_id, raised_by));
}

pub fn emit_dispute_resolved(
    env: &Env,
    trade_id: u64,
    resolution: DisputeResolution,
    recipient: Address,
) {
    env.events()
        .publish((symbol_short!("resolved"),), (trade_id, resolution, recipient));
}

pub fn emit_trade_cancelled(env: &Env, trade_id: u64) {
    env.events()
        .publish((symbol_short!("cancel"),), trade_id);
}

pub fn emit_arbitrator_registered(env: &Env, arbitrator: Address) {
    env.events()
        .publish((symbol_short!("arb_reg"),), arbitrator);
}

pub fn emit_arbitrator_removed(env: &Env, arbitrator: Address) {
    env.events()
        .publish((symbol_short!("arb_rem"),), arbitrator);
}

pub fn emit_fee_updated(env: &Env, fee_bps: u32) {
    env.events().publish((symbol_short!("fee_upd"),), fee_bps);
}

pub fn emit_fees_withdrawn(env: &Env, amount: u64, to: Address) {
    env.events()
        .publish((symbol_short!("fees_out"),), (amount, to));
}
