/// Current event schema version. Bump when payload fields change.
pub const EVENT_VERSION: u32 = 2;

use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol};

use crate::types::{DisputeResolution, SubscriptionTier, UserTier};

// ---------------------------------------------------------------------------
// Event categories (used as the first topic for indexer filtering)
// ---------------------------------------------------------------------------
//
// Topic layout: (category, event_name)
// This lets indexers subscribe to a whole category (e.g. "trade") or a
// specific event (e.g. "trade" + "created") without scanning all events.

fn cat_trade() -> Symbol { symbol_short!("trade") }
fn cat_arb()   -> Symbol { symbol_short!("arb") }
fn cat_fee()   -> Symbol { symbol_short!("fee") }
fn cat_tmpl()  -> Symbol { symbol_short!("tmpl") }
fn cat_sub()   -> Symbol { symbol_short!("sub") }
fn cat_gov()   -> Symbol { symbol_short!("gov") }
fn cat_sys()   -> Symbol { symbol_short!("sys") }
fn cat_ins()   -> Symbol { symbol_short!("ins") }

// ---------------------------------------------------------------------------
// Structured event payloads
// ---------------------------------------------------------------------------

#[contracttype] #[derive(Clone, Debug)]
pub struct EvTradeCreated   { pub v: u32, pub trade_id: u64, pub seller: Address, pub buyer: Address, pub amount: u64, pub currency: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvTradeFunded    { pub v: u32, pub trade_id: u64 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvTradeCompleted { pub v: u32, pub trade_id: u64 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvTradeConfirmed { pub v: u32, pub trade_id: u64, pub payout: u64, pub fee: u64 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvTradeCancelled { pub v: u32, pub trade_id: u64 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvTimeReleased   { pub v: u32, pub trade_id: u64, pub seller: Address, pub payout: u64 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvMetaUpdated    { pub v: u32, pub trade_id: u64 }

#[contracttype] #[derive(Clone, Debug)]
pub struct EvDisputeRaised  { pub v: u32, pub trade_id: u64, pub raised_by: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvDisputeResolved { pub v: u32, pub trade_id: u64, pub resolution: DisputeResolution, pub recipient: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvPartialResolved { pub v: u32, pub trade_id: u64, pub buyer_amount: u64, pub seller_amount: u64, pub fee: u64 }

#[contracttype] #[derive(Clone, Debug)]
pub struct EvArbRegistered  { pub v: u32, pub arbitrator: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvArbRemoved     { pub v: u32, pub arbitrator: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvArbRated       { pub v: u32, pub arbitrator: Address, pub trade_id: u64, pub rater: Address, pub stars: u32 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvArbRepUpdated  { pub v: u32, pub arbitrator: Address, pub resolved: u32, pub rating_sum: u32, pub rating_count: u32 }

#[contracttype] #[derive(Clone, Debug)]
pub struct EvFeeUpdated     { pub v: u32, pub fee_bps: u32 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvFeesWithdrawn  { pub v: u32, pub amount: u64, pub to: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvFeesDistributed { pub v: u32, pub to: Address, pub amount: u64 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvCustomFeeSet   { pub v: u32, pub user: Address, pub fee_bps: u32 }

#[contracttype] #[derive(Clone, Debug)]
pub struct EvTierUpgraded   { pub v: u32, pub user: Address, pub tier: UserTier }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvTierDowngraded { pub v: u32, pub user: Address, pub tier: UserTier }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvTierConfigUpdated { pub v: u32 }

#[contracttype] #[derive(Clone, Debug)]
pub struct EvTemplateCreated     { pub v: u32, pub template_id: u64, pub owner: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvTemplateUpdated     { pub v: u32, pub template_id: u64, pub version: u32 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvTemplateDeactivated { pub v: u32, pub template_id: u64 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvTemplateTrade       { pub v: u32, pub trade_id: u64, pub template_id: u64, pub version: u32 }

#[contracttype] #[derive(Clone, Debug)]
pub struct EvSubscribed           { pub v: u32, pub subscriber: Address, pub tier: SubscriptionTier, pub expires_at: u32 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvSubscriptionRenewed  { pub v: u32, pub subscriber: Address, pub tier: SubscriptionTier, pub expires_at: u32 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvSubscriptionCancelled { pub v: u32, pub subscriber: Address }

#[contracttype] #[derive(Clone, Debug)]
pub struct EvGovTokenInitialized { pub v: u32, pub token: Address, pub initial_holder: Address, pub supply: i128 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvProposalCreated  { pub v: u32, pub proposal_id: u64, pub proposer: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvVoteCast         { pub v: u32, pub proposal_id: u64, pub voter: Address, pub support: bool, pub weight: i128 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvProposalExecuted { pub v: u32, pub proposal_id: u64 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvDelegated        { pub v: u32, pub delegator: Address, pub delegatee: Address }

#[contracttype] #[derive(Clone, Debug)]
pub struct EvPaused           { pub v: u32, pub admin: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvUnpaused         { pub v: u32, pub admin: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvEmergencyWithdraw { pub v: u32, pub to: Address, pub amount: u64 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvUpgraded         { pub v: u32, pub new_version: u32 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvMigrated         { pub v: u32, pub from_version: u32, pub to_version: u32 }

#[contracttype] #[derive(Clone, Debug)]
pub struct EvPrivacySet        { pub v: u32, pub trade_id: u64 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvDisclosureGranted { pub v: u32, pub trade_id: u64, pub grantee: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvDisclosureRevoked { pub v: u32, pub trade_id: u64, pub grantee: Address }

#[contracttype] #[derive(Clone, Debug)]
pub struct EvBridgeOracleSet        { pub v: u32, pub oracle: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvBridgeTradeCreated     { pub v: u32, pub trade_id: u64, pub source_chain: String }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvBridgeDepositConfirmed { pub v: u32, pub trade_id: u64 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvBridgeTradeExpired     { pub v: u32, pub trade_id: u64 }

#[contracttype] #[derive(Clone, Debug)]
pub struct EvInsProviderRegistered { pub v: u32, pub provider: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvInsProviderRemoved    { pub v: u32, pub provider: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvInsPurchased          { pub v: u32, pub trade_id: u64, pub provider: Address, pub premium: u64, pub coverage: u64 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvInsClaimed            { pub v: u32, pub trade_id: u64, pub payout: u64, pub recipient: Address }

// ---------------------------------------------------------------------------
// Emit helpers — topic: (category, event_name)
// ---------------------------------------------------------------------------

pub fn emit_trade_created(env: &Env, trade_id: u64, seller: Address, buyer: Address, amount: u64, currency: Address) {
    env.events().publish((cat_trade(), symbol_short!("created")), EvTradeCreated { v: EVENT_VERSION, trade_id, seller, buyer, amount, currency });
}

pub fn emit_compliance_failed(env: &Env, user: Address, reason: &soroban_sdk::String) {
    env.events().publish((symbol_short!("compl_fail"),), (user, reason.clone()));
}

pub fn emit_compliance_passed(env: &Env, trade_id: u64, seller: Address, buyer: Address, amount: u64) {
    env.events().publish((symbol_short!("compl_pass"),), (trade_id, seller, buyer, amount));
}

pub fn emit_trade_funded(env: &Env, trade_id: u64) {
    env.events().publish((cat_trade(), symbol_short!("funded")), EvTradeFunded { v: EVENT_VERSION, trade_id });
}
pub fn emit_trade_completed(env: &Env, trade_id: u64) {
    env.events().publish((cat_trade(), symbol_short!("complete")), EvTradeCompleted { v: EVENT_VERSION, trade_id });
}
pub fn emit_trade_confirmed(env: &Env, trade_id: u64, payout: u64, fee: u64) {
    env.events().publish((cat_trade(), symbol_short!("confirm")), EvTradeConfirmed { v: EVENT_VERSION, trade_id, payout, fee });
}
pub fn emit_trade_cancelled(env: &Env, trade_id: u64) {
    env.events().publish((cat_trade(), symbol_short!("cancel")), EvTradeCancelled { v: EVENT_VERSION, trade_id });
}
pub fn emit_time_released(env: &Env, trade_id: u64, seller: Address, payout: u64) {
    env.events().publish((cat_trade(), symbol_short!("time_rel")), EvTimeReleased { v: EVENT_VERSION, trade_id, seller, payout });
}
pub fn emit_metadata_updated(env: &Env, trade_id: u64) {
    env.events().publish((cat_trade(), symbol_short!("meta_upd")), EvMetaUpdated { v: EVENT_VERSION, trade_id });
}

pub fn emit_dispute_raised(env: &Env, trade_id: u64, raised_by: Address) {
    env.events().publish((cat_trade(), symbol_short!("dispute")), EvDisputeRaised { v: EVENT_VERSION, trade_id, raised_by });
}
pub fn emit_dispute_resolved(env: &Env, trade_id: u64, resolution: DisputeResolution, recipient: Address) {
    env.events().publish((cat_trade(), symbol_short!("resolved")), EvDisputeResolved { v: EVENT_VERSION, trade_id, resolution, recipient });
}
pub fn emit_partial_resolved(env: &Env, trade_id: u64, buyer_amount: u64, seller_amount: u64, fee: u64) {
    env.events().publish((cat_trade(), symbol_short!("part_res")), EvPartialResolved { v: EVENT_VERSION, trade_id, buyer_amount, seller_amount, fee });
}

pub fn emit_arbitrator_registered(env: &Env, arbitrator: Address) {
    env.events().publish((cat_arb(), symbol_short!("arb_reg")), EvArbRegistered { v: EVENT_VERSION, arbitrator });
}
pub fn emit_arbitrator_removed(env: &Env, arbitrator: Address) {
    env.events().publish((cat_arb(), symbol_short!("arb_rem")), EvArbRemoved { v: EVENT_VERSION, arbitrator });
}
pub fn emit_arb_rated(env: &Env, arbitrator: Address, trade_id: u64, rater: Address, stars: u32) {
    env.events().publish((cat_arb(), symbol_short!("arb_rate")), EvArbRated { v: EVENT_VERSION, arbitrator, trade_id, rater, stars });
}
pub fn emit_arb_rep_updated(env: &Env, arbitrator: Address, resolved_count: u32, rating_sum: u32, rating_count: u32) {
    env.events().publish((cat_arb(), symbol_short!("arb_rep")), EvArbRepUpdated { v: EVENT_VERSION, arbitrator, resolved: resolved_count, rating_sum, rating_count });
}

pub fn emit_fee_updated(env: &Env, fee_bps: u32) {
    env.events().publish((cat_fee(), symbol_short!("fee_upd")), EvFeeUpdated { v: EVENT_VERSION, fee_bps });
}
pub fn emit_fees_withdrawn(env: &Env, amount: u64, to: Address) {
    env.events().publish((cat_fee(), symbol_short!("fees_out")), EvFeesWithdrawn { v: EVENT_VERSION, amount, to });
}
pub fn emit_fees_distributed(env: &Env, to: Address, amount: u64) {
    env.events().publish((cat_fee(), symbol_short!("fee_dst")), EvFeesDistributed { v: EVENT_VERSION, to, amount });
}
pub fn emit_custom_fee_set(env: &Env, user: Address, fee_bps: u32) {
    env.events().publish((cat_fee(), symbol_short!("cust_fee")), EvCustomFeeSet { v: EVENT_VERSION, user, fee_bps });
}

pub fn emit_tier_upgraded(env: &Env, user: Address, new_tier: UserTier) {
    env.events().publish((cat_fee(), symbol_short!("tier_up")), EvTierUpgraded { v: EVENT_VERSION, user, tier: new_tier });
}
pub fn emit_tier_downgraded(env: &Env, user: Address, new_tier: UserTier) {
    env.events().publish((cat_fee(), symbol_short!("tier_dn")), EvTierDowngraded { v: EVENT_VERSION, user, tier: new_tier });
}
pub fn emit_tier_config_updated(env: &Env) {
    env.events().publish((cat_fee(), symbol_short!("tier_cfg")), EvTierConfigUpdated { v: EVENT_VERSION });
}

pub fn emit_template_created(env: &Env, template_id: u64, owner: Address) {
    env.events().publish((cat_tmpl(), symbol_short!("tmpl_cr")), EvTemplateCreated { v: EVENT_VERSION, template_id, owner });
}
pub fn emit_template_updated(env: &Env, template_id: u64, version: u32) {
    env.events().publish((cat_tmpl(), symbol_short!("tmpl_up")), EvTemplateUpdated { v: EVENT_VERSION, template_id, version });
}
pub fn emit_template_deactivated(env: &Env, template_id: u64) {
    env.events().publish((cat_tmpl(), symbol_short!("tmpl_off")), EvTemplateDeactivated { v: EVENT_VERSION, template_id });
}
pub fn emit_template_trade(env: &Env, trade_id: u64, template_id: u64, version: u32) {
    env.events().publish((cat_tmpl(), symbol_short!("tmpl_tr")), EvTemplateTrade { v: EVENT_VERSION, trade_id, template_id, version });
}

pub fn emit_subscribed(env: &Env, subscriber: Address, tier: SubscriptionTier, expires_at: u32) {
    env.events().publish((cat_sub(), symbol_short!("sub_new")), EvSubscribed { v: EVENT_VERSION, subscriber, tier, expires_at });
}
pub fn emit_subscription_renewed(env: &Env, subscriber: Address, tier: SubscriptionTier, expires_at: u32) {
    env.events().publish((cat_sub(), symbol_short!("sub_ren")), EvSubscriptionRenewed { v: EVENT_VERSION, subscriber, tier, expires_at });
}
pub fn emit_subscription_cancelled(env: &Env, subscriber: Address) {
    env.events().publish((cat_sub(), symbol_short!("sub_can")), EvSubscriptionCancelled { v: EVENT_VERSION, subscriber });
}

pub fn emit_proposal_created(env: &Env, proposal_id: u64, proposer: Address) {
    env.events().publish((cat_gov(), symbol_short!("prop_cr")), EvProposalCreated { v: EVENT_VERSION, proposal_id, proposer });
}
pub fn emit_vote_cast(env: &Env, proposal_id: u64, voter: Address, support: bool, weight: i128) {
    env.events().publish((cat_gov(), symbol_short!("voted")), EvVoteCast { v: EVENT_VERSION, proposal_id, voter, support, weight });
}
pub fn emit_proposal_executed(env: &Env, proposal_id: u64) {
    env.events().publish((cat_gov(), symbol_short!("prop_ex")), EvProposalExecuted { v: EVENT_VERSION, proposal_id });
}
pub fn emit_delegated(env: &Env, delegator: Address, delegatee: Address) {
    env.events().publish((cat_gov(), symbol_short!("delegat")), EvDelegated { v: EVENT_VERSION, delegator, delegatee });
}

pub fn emit_gov_token_initialized(env: &Env, token: Address, initial_holder: Address, supply: i128) {
    env.events().publish((cat_gov(), symbol_short!("gov_init")), EvGovTokenInitialized { v: EVENT_VERSION, token, initial_holder, supply });
}

pub fn emit_paused(env: &Env, admin: Address) {
    env.events().publish((cat_sys(), symbol_short!("paused")), EvPaused { v: EVENT_VERSION, admin });
}
pub fn emit_unpaused(env: &Env, admin: Address) {
    env.events().publish((cat_sys(), symbol_short!("unpaused")), EvUnpaused { v: EVENT_VERSION, admin });
}
pub fn emit_emergency_withdraw(env: &Env, to: Address, amount: u64) {
    env.events().publish((cat_sys(), symbol_short!("emrg_wd")), EvEmergencyWithdraw { v: EVENT_VERSION, to, amount });
}
pub fn emit_upgraded(env: &Env, new_version: u32) {
    env.events().publish((cat_sys(), symbol_short!("upgraded")), EvUpgraded { v: EVENT_VERSION, new_version });
}
pub fn emit_migrated(env: &Env, from_version: u32, to_version: u32) {
    env.events().publish((cat_sys(), symbol_short!("migrated")), EvMigrated { v: EVENT_VERSION, from_version, to_version });
}
pub fn emit_privacy_set(env: &Env, trade_id: u64) {
    env.events().publish((cat_sys(), symbol_short!("priv_set")), EvPrivacySet { v: EVENT_VERSION, trade_id });
}
pub fn emit_disclosure_granted(env: &Env, trade_id: u64, grantee: Address) {
    env.events().publish((cat_sys(), symbol_short!("disc_gr")), EvDisclosureGranted { v: EVENT_VERSION, trade_id, grantee });
}
pub fn emit_disclosure_revoked(env: &Env, trade_id: u64, grantee: Address) {
    env.events().publish((cat_sys(), symbol_short!("disc_rv")), EvDisclosureRevoked { v: EVENT_VERSION, trade_id, grantee });
}

pub fn emit_bridge_oracle_set(env: &Env, oracle: Address) {
    env.events().publish((cat_sys(), symbol_short!("brg_set")), EvBridgeOracleSet { v: EVENT_VERSION, oracle });
}
pub fn emit_bridge_trade_created(env: &Env, trade_id: u64, source_chain: String) {
    env.events().publish((cat_sys(), symbol_short!("brg_cr")), EvBridgeTradeCreated { v: EVENT_VERSION, trade_id, source_chain });
}
pub fn emit_bridge_deposit_confirmed(env: &Env, trade_id: u64) {
    env.events().publish((cat_sys(), symbol_short!("brg_ok")), EvBridgeDepositConfirmed { v: EVENT_VERSION, trade_id });
}
pub fn emit_bridge_trade_expired(env: &Env, trade_id: u64) {
    env.events().publish((cat_sys(), symbol_short!("brg_exp")), EvBridgeTradeExpired { v: EVENT_VERSION, trade_id });
}

pub fn emit_insurance_provider_registered(env: &Env, provider: Address) {
    env.events().publish((cat_ins(), symbol_short!("ins_reg")), EvInsProviderRegistered { v: EVENT_VERSION, provider });
}
pub fn emit_insurance_provider_removed(env: &Env, provider: Address) {
    env.events().publish((cat_ins(), symbol_short!("ins_rem")), EvInsProviderRemoved { v: EVENT_VERSION, provider });
}
pub fn emit_insurance_purchased(env: &Env, trade_id: u64, provider: Address, premium: u64, coverage: u64) {
    env.events().publish((cat_ins(), symbol_short!("ins_buy")), EvInsPurchased { v: EVENT_VERSION, trade_id, provider, premium, coverage });
}
pub fn emit_insurance_claimed(env: &Env, trade_id: u64, payout: u64, recipient: Address) {
    env.events().publish((cat_ins(), symbol_short!("ins_pay")), EvInsClaimed { v: EVENT_VERSION, trade_id, payout, recipient });
}

// ---------------------------------------------------------------------------
// Oracle events
// ---------------------------------------------------------------------------

fn cat_oracle() -> Symbol { symbol_short!("oracle") }

#[contracttype] #[derive(Clone, Debug)]
pub struct EvOracleRegistered { pub v: u32, pub base: Address, pub quote: Address, pub oracle: Address, pub priority: u32 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvOracleRemoved    { pub v: u32, pub base: Address, pub quote: Address, pub oracle: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvOraclePriceFetched { pub v: u32, pub base: Address, pub quote: Address, pub price: i128, pub decimals: u32 }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvOracleUnavailable  { pub v: u32, pub base: Address, pub quote: Address }

pub fn emit_oracle_registered(env: &Env, base: Address, quote: Address, oracle: Address, priority: u32) {
    env.events().publish((cat_oracle(), symbol_short!("orc_reg")), EvOracleRegistered { v: EVENT_VERSION, base, quote, oracle, priority });
}
pub fn emit_oracle_removed(env: &Env, base: Address, quote: Address, oracle: Address) {
    env.events().publish((cat_oracle(), symbol_short!("orc_rem")), EvOracleRemoved { v: EVENT_VERSION, base, quote, oracle });
}
pub fn emit_oracle_price_fetched(env: &Env, base: Address, quote: Address, price: i128, decimals: u32) {
    env.events().publish((cat_oracle(), symbol_short!("orc_px")), EvOraclePriceFetched { v: EVENT_VERSION, base, quote, price, decimals });
}
pub fn emit_oracle_unavailable(env: &Env, base: Address, quote: Address) {
    env.events().publish((cat_oracle(), symbol_short!("orc_err")), EvOracleUnavailable { v: EVENT_VERSION, base, quote });
}

// ---------------------------------------------------------------------------
// Upgrade system events
// ---------------------------------------------------------------------------

#[contracttype] #[derive(Clone, Debug)]
pub struct EvUpgradeProposed   { pub v: u32, pub proposed_by: Address, pub executable_after: u32, pub description: String }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvUpgradeCancelled  { pub v: u32, pub cancelled_by: Address }
#[contracttype] #[derive(Clone, Debug)]
pub struct EvUpgradeRolledBack { pub v: u32, pub rolled_back_by: Address, pub restored_version: u32 }

pub fn emit_upgrade_proposed(env: &Env, proposed_by: Address, executable_after: u32, description: String) {
    env.events().publish((cat_sys(), symbol_short!("up_prop")), EvUpgradeProposed { v: EVENT_VERSION, proposed_by, executable_after, description });
}
pub fn emit_upgrade_cancelled(env: &Env, cancelled_by: Address) {
    env.events().publish((cat_sys(), symbol_short!("up_can")), EvUpgradeCancelled { v: EVENT_VERSION, cancelled_by });
}
pub fn emit_upgrade_rolled_back(env: &Env, rolled_back_by: Address, restored_version: u32) {
    env.events().publish((cat_sys(), symbol_short!("up_rb")), EvUpgradeRolledBack { v: EVENT_VERSION, rolled_back_by, restored_version });
}
