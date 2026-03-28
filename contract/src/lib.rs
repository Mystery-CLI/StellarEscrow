#![no_std]

mod analytics;
mod amm;
mod errors;
mod events;
mod governance;
mod multisig;
mod oracle;
mod privacy;
mod queries;
mod reputation;
mod social;
mod storage;
mod subscription;
mod templates;
mod tiers;
mod types;
mod upgrade;
mod proxy;

use soroban_sdk::{contract, contractimpl, token::TokenClient, Address, BytesN, Env};

pub use errors::ContractError;
pub use types::{
    ArbitrationConfig, ArbitratorVote, DisclosureGrant, DisputeResolution, MultiSigConfig,
    Proposal, ProposalAction, ProposalStatus, Subscription, SubscriptionTier, TierConfig,
    TemplateTerms, TemplateVersion, Trade, TradePrivacy, TradeStatus, TradeTemplate,
    UserTier, UserTierInfo, VotingSummary,
    ArbitratorReputation, DisclosureGrant, DisputeResolution, Proposal, ProposalAction,
    ProposalStatus, Subscription, SubscriptionTier, TierConfig, TemplateTerms, TemplateVersion,
    Trade, TradePrivacy, TradeStatus, TradeTemplate, UserTier, UserTierInfo,
};
pub use queries::{PageParams, SortDirection, TradeFilter, TradeSortField, TradeStats};
pub use oracle::{OracleEntry, PriceData, PriceValidation};
pub use upgrade::{RollbackSnapshot, UpgradeProposal};
pub use proxy::*;

use storage::{
    get_accumulated_fees, get_admin, get_fee_bps, get_trade, get_usdc_token,
    has_arbitrator, has_initialized, increment_trade_counter, is_initialized, is_paused,
    remove_arbitrator, save_arbitrator, save_trade, set_accumulated_fees, set_admin, set_fee_bps,
    ArbitratorReputation, DisputeResolution, TierConfig, TemplateTerms,
    TemplateVersion, Trade, TradeStatus, TradeTemplate, UserTier, UserTierInfo,
};

use storage::{
    get_accumulated_fees, get_admin, get_fee_bps, get_trade, get_trade_counter, get_usdc_token,
    has_arbitrator, has_initialized, has_rated, increment_trade_counter, is_initialized, is_paused,
    mark_rated, remove_arbitrator, save_arbitrator, save_arbitrator_reputation, save_trade,
    set_accumulated_fees, set_admin, set_fee_bps, set_initialized, set_paused, set_trade_counter,
    set_usdc_token,
    CrossChainInfo, DisputeResolution, InsurancePolicy,
    TierConfig, TemplateTerms, TemplateVersion, Trade, TradeStatus,
    TradeTemplate, UserTier, UserTierInfo,
};

use storage::{
    add_accumulated_fees, get_accumulated_fees, get_admin, get_currency_fees, get_fee_bps,
    get_trade, get_trade_counter, get_usdc_token, has_arbitrator, has_initialized, has_rated,
    increment_trade_counter, is_initialized, is_paused, mark_rated, remove_arbitrator,
    save_arbitrator, save_arbitrator_reputation, save_trade, set_accumulated_fees, set_admin,
    set_currency_fees, set_fee_bps, set_initialized, set_paused, set_trade_counter,
    set_usdc_token, CrossChainInfo, InsurancePolicy,
};

fn token_client<'a>(env: &'a Env, token: &Address) -> token::Client<'a> {
    token::Client::new(env, token)
}

/// Maximum length of JSON metadata string
pub const MAX_METADATA_SIZE: usize = 1024;

fn validate_metadata(env: &Env, meta: &Option<soroban_sdk::String>) -> Result<(), ContractError> {
    if let Some(m) = meta {
        if m.len() > MAX_METADATA_SIZE as u32 {
            return Err(ContractError::MetadataValueTooLong);
        }
        if m.len() == 0 {
            return Err(ContractError::InvalidMetadata);
        }

        // Try to parse using serde_json. 
        // Need to convert soroban_sdk::String to alloc::string::String first
        extern crate alloc;
        use alloc::string::String as AllocString;
        use alloc::vec::Vec as AllocVec;

        let mut out = AllocVec::new();
        let mut slice = [0u8; 1024];
        let len = m.len() as usize;
        m.copy_into_slice(&mut slice[..len]);
        out.extend_from_slice(&slice[..len]);

        let json_str = AllocString::from_utf8(out).map_err(|_| ContractError::InvalidMetadata)?;
        serde_json::from_str::<serde_json::Value>(&json_str).map_err(|_| ContractError::InvalidMetadata)?;
    }
    Ok(())
}

#[inline]
fn require_initialized(env: &Env) -> Result<(), ContractError> {
    if !is_initialized(env) {
        return Err(ContractError::NotInitialized);
    }
    Ok(())
}

#[inline]
fn require_not_paused(env: &Env) -> Result<(), ContractError> {
    if is_paused(env) {
        return Err(ContractError::ContractPaused);
    }
    Ok(())
}

/// Shared fee calculation to avoid duplication.
#[inline]
fn calc_fee(env: &Env, seller: &Address, amount: u64) -> Result<u64, ContractError> {
    let fee_bps = get_fee_bps(env)?;
    let effective_bps = tiers::effective_fee_bps(env, seller, fee_bps);
    amount
        .checked_mul(effective_bps as u64)
        .ok_or(ContractError::Overflow)?
        .checked_div(10000)
        .ok_or(ContractError::Overflow)
}



fn require_admin(env: &Env, admin: &Address) -> Result<(), ContractError> {
    let current_admin = get_admin(env)?;
    if &current_admin != admin {
        return Err(ContractError::Unauthorized);
    }
    admin.require_auth();
    Ok(())
}

fn validate_user_compliance(env: &Env, user: &Address, amount: u64) -> Result<(), ContractError> {
    let comp = storage::get_user_compliance(env, user).ok_or(ContractError::KycNotVerified)?;
    if comp.kyc_status != crate::types::KycStatus::Verified {
        events::emit_compliance_failed(env, user.clone(), &soroban_sdk::String::from_str(env, "KYC_NOT_VERIFIED"));
        return Err(ContractError::KycNotVerified);
    }
    if !comp.aml_cleared {
        events::emit_compliance_failed(env, user.clone(), &soroban_sdk::String::from_str(env, "AML_NOT_CLEARED"));
        return Err(ContractError::AmlNotCleared);
    }
    if !storage::is_jurisdiction_allowed(env, &comp.jurisdiction) {
        events::emit_compliance_failed(env, user.clone(), &soroban_sdk::String::from_str(env, "JURISDICTION_BLOCKED"));
        return Err(ContractError::JurisdictionRestricted);
    }
    let user_limit = storage::get_user_trade_limit(env, user);
    if user_limit > 0 && amount > user_limit {
        events::emit_compliance_failed(env, user.clone(), &soroban_sdk::String::from_str(env, "USER_LIMIT_EXCEEDED"));
        return Err(ContractError::TradeAmountLimitExceeded);
    }
    let global_limit = storage::get_global_trade_limit(env);
    if amount > global_limit {
        events::emit_compliance_failed(env, user.clone(), &soroban_sdk::String::from_str(env, "GLOBAL_LIMIT_EXCEEDED"));
        return Err(ContractError::TradeAmountLimitExceeded);
    }
    Ok(())
}

#[contract]
pub struct StellarEscrowContract;

#[contractimpl]
impl StellarEscrowContract {
    /// Initialize the contract with admin, USDC token address, and platform fee
    pub fn initialize(env: Env, admin: Address, usdc_token: Address, fee_bps: u32) -> Result<(), ContractError> {
        if is_initialized(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        if fee_bps > 10000 {
            return Err(ContractError::InvalidFeeBps);
        }
        admin.require_auth();
        set_admin(&env, &admin);
        set_usdc_token(&env, &usdc_token);
        set_fee_bps(&env, fee_bps);
        set_trade_counter(&env, 0);
        set_accumulated_fees(&env, 0);
        set_initialized(&env);
        set_version(&env, 1);
        Ok(())
    }

    /// Register an arbitrator (admin only)
    pub fn register_arbitrator(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        save_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_registered(&env, arbitrator);
        Ok(())
    }

    /// Remove an arbitrator (admin only)
    pub fn remove_arbitrator_fn(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        remove_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_removed(&env, arbitrator);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Multi-Signature Arbitration
    // -------------------------------------------------------------------------

    /// Create a trade with multi-signature arbitration.
    /// All arbitrators in `config` must be registered; threshold must be > 0
    /// and ≤ arbitrators count.
    pub fn create_multisig_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        config: MultiSigConfig,
        expiry_time: Option<u64>,
        currency: Option<Address>,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        if config.threshold == 0 || config.threshold > config.arbitrators.len() {
            return Err(ContractError::InvalidMultiSigConfig);
        }
        for i in 0..config.arbitrators.len() {
            if !has_arbitrator(&env, &config.arbitrators.get(i).unwrap()) {
                return Err(ContractError::ArbitratorNotRegistered);
            }
        }
        if let Some(expiry) = expiry_time {
            if expiry <= env.ledger().timestamp() {
                return Err(ContractError::InvalidExpiry);
            }
        }
        seller.require_auth();
        let token = currency.unwrap_or(get_usdc_token(&env)?);
        let trade_id = increment_trade_counter(&env)?;
        let fee = calc_fee(&env, &seller, amount)?;
        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee,
            arbitrator: Some(ArbitrationConfig::MultiSig(config)),
            status: TradeStatus::Created,
            expiry_time,
            currency: token,
            metadata: None,
        };
        save_trade(&env, trade_id, &trade);
        events::emit_trade_created(&env, trade_id, seller, buyer, amount, trade.currency);
        Ok(trade_id)
    }

    /// Cast a vote on a disputed multi-sig trade.
    pub fn cast_vote(
        env: Env,
        trade_id: u64,
        arbitrator: Address,
        resolution: DisputeResolution,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        multisig::cast_vote(&env, trade_id, &arbitrator, resolution)
    }

    /// Return the current voting state for a multi-sig trade.
    pub fn get_voting_summary(env: Env, trade_id: u64) -> Result<VotingSummary, ContractError> {
        require_initialized(&env)?;
        multisig::voting_summary(&env, trade_id)
    }

    /// Force-resolve a multi-sig dispute after the voting window expires without
    /// consensus. Defaults to refunding the buyer. Admin only.
    pub fn resolve_expired_dispute(
        env: Env,
        admin: Address,
        trade_id: u64,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        require_admin(&env, &admin)?;
        let resolution = multisig::resolve_expired_dispute(&env, trade_id, &admin)?;
        let trade = get_trade(&env, trade_id)?;
        StellarEscrowContract::execute_dispute_resolution(env, trade_id, resolution, trade)
    // Arbitrator Reputation
    // -------------------------------------------------------------------------

    /// Rate the arbitrator of a disputed trade (buyer or seller, once each).
    pub fn rate_arbitrator(
        env: Env,
        trade_id: u64,
        rater: Address,
        stars: u32,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let trade = get_trade(&env, trade_id)?;
        let arbitrator = match &trade.arbitrator {
            Some(arb) => arb.clone(),
            None => return Err(ContractError::NoArbitrator),
        };
        rater.require_auth();
        reputation::rate_arbitrator(
            &env,
            trade_id,
            &rater,
            &arbitrator,
            &trade.buyer,
            &trade.seller,
            &trade.status,
            stars,
        )
    }

    /// Raw reputation record for an arbitrator.
    pub fn get_arbitrator_reputation(env: Env, arbitrator: Address) -> ArbitratorReputation {
        storage::get_arbitrator_reputation(&env, &arbitrator)
    }

    /// Average star rating ×100 (e.g. 450 = 4.50 stars). Returns 0 if unrated.
    pub fn get_arbitrator_avg_rating(env: Env, arbitrator: Address) -> u32 {
        reputation::average_rating_x100(&storage::get_arbitrator_reputation(&env, &arbitrator))
    }

    /// Resolution rate in basis points (0–10000).
    pub fn get_arbitrator_resolution_rate(env: Env, arbitrator: Address) -> u32 {
        reputation::resolution_rate_bps(&storage::get_arbitrator_reputation(&env, &arbitrator))
    }

    /// Composite reputation score (0–10000).
    pub fn get_arbitrator_score(env: Env, arbitrator: Address) -> u32 {
        reputation::composite_score(&storage::get_arbitrator_reputation(&env, &arbitrator))
    }

    /// From a candidate list, return the registered arbitrator with the highest score.
    pub fn select_best_arbitrator(
        env: Env,
        candidates: soroban_sdk::Vec<Address>,
    ) -> Result<Address, ContractError> {
        require_initialized(&env)?;
        reputation::select_best_arbitrator(&env, &candidates)
    }

    /// Reputation records for all arbitrators in the supplied list (same order).
    pub fn get_arbitrator_reputations(
        env: Env,
        arbitrators: soroban_sdk::Vec<Address>,
    ) -> soroban_sdk::Vec<ArbitratorReputation> {
        reputation::get_reputations(&env, &arbitrators)
    }

    /// Update platform fee (admin only)
    pub fn update_fee(env: Env, fee_bps: u32) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if fee_bps > 10000 {
            return Err(ContractError::InvalidFeeBps);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_fee_bps(&env, fee_bps);
        events::emit_fee_updated(&env, fee_bps);
        Ok(())
    }

    pub fn set_user_compliance(
        env: Env,
        admin: Address,
        user: Address,
        compliance: crate::types::UserCompliance,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        require_admin(&env, &admin)?;
        storage::save_user_compliance(&env, &user, &compliance);
        Ok(())
    }

    pub fn get_user_compliance(env: Env, user: Address) -> Option<crate::types::UserCompliance> {
        storage::get_user_compliance(&env, &user)
    }

    pub fn set_user_trade_limit(env: Env, admin: Address, user: Address, limit: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        require_admin(&env, &admin)?;
        storage::set_user_trade_limit(&env, &user, limit);
        Ok(())
    }

    pub fn get_user_trade_limit(env: Env, user: Address) -> u64 {
        storage::get_user_trade_limit(&env, &user)
    }

    pub fn set_jurisdiction_rule(
        env: Env,
        admin: Address,
        jurisdiction: soroban_sdk::String,
        allowed: bool,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        require_admin(&env, &admin)?;
        storage::set_jurisdiction_rule(&env, &jurisdiction, allowed);
        Ok(())
    }

    pub fn is_jurisdiction_allowed(env: Env, jurisdiction: soroban_sdk::String) -> bool {
        storage::is_jurisdiction_allowed(&env, &jurisdiction)
    }

    pub fn set_global_trade_limit(env: Env, admin: Address, limit: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        require_admin(&env, &admin)?;
        storage::set_global_trade_limit(&env, limit);
        Ok(())
    }

    pub fn get_global_trade_limit(env: Env) -> u64 {
        storage::get_global_trade_limit(&env)
    }

    /// Withdraw accumulated fees for USDC (admin only) — backward-compatible
    pub fn withdraw_fees(env: Env, to: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        let token = get_usdc_token(&env)?;
        Self::withdraw_fees_for_currency_inner(&env, &admin, &token, to)
    }

    /// Withdraw accumulated fees for a specific currency (admin only)
    pub fn withdraw_fees_for_currency(env: Env, currency: Address, to: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        Self::withdraw_fees_for_currency_inner(&env, &admin, &currency, to)
    }

    fn withdraw_fees_for_currency_inner(
        env: &Env,
        _admin: &Address,
        currency: &Address,
        to: Address,
    ) -> Result<(), ContractError> {
        let fees = get_currency_fees(env, currency);
        if fees == 0 {
            return Err(ContractError::NoFeesToWithdraw);
        }
        let token_client = token::Client::new(env, currency);
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &to, &(fees as i128));
        set_currency_fees(env, currency, 0);
        // Keep legacy ACCUMULATED_FEES in sync when withdrawing USDC
        // (best-effort; callers should prefer get_fees_for_currency)
        events::emit_fees_withdrawn(env, fees, to);
        Ok(())
    }

    /// Create a new trade with optional metadata and optional time lock
    pub fn create_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        arbitrator: Option<Address>,
        expiry_time: Option<u64>,
        currency: Option<Address>,
        metadata: Option<TradeMetadata>,
        metadata: OptionalMetadata,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        // expiry_time must be in the future (Stellar ledger time is UTC seconds)
        if let Some(expiry) = expiry_time {
            let now = env.ledger().timestamp();
            if expiry <= now {
                return Err(ContractError::InvalidExpiry);
            }
        }
        seller.require_auth();
        validate_user_compliance(&env, &seller, amount)?;
        validate_user_compliance(&env, &buyer, amount)?;
        if let Some(ref arb) = arbitrator {
            if !has_arbitrator(&env, arb) {
                return Err(ContractError::ArbitratorNotRegistered);
            }
        }
        if let Some(ref meta) = metadata {
            validate_metadata(meta)?;
        }
        // Default to USDC when no currency specified (backward compat)
        let token = currency.unwrap_or(get_usdc_token(&env)?);
        validate_metadata(&metadata)?;
        let trade_id = increment_trade_counter(&env)?;
        let fee_bps = get_fee_bps(&env)?;
        let effective_bps = tiers::effective_fee_bps(&env, &seller, fee_bps);
        let discount = subscription::subscription_discount_bps(&env, &seller);
        let final_bps = effective_bps.saturating_sub(discount);
        let fee = amount
            .checked_mul(final_bps as u64)
            .ok_or(ContractError::Overflow)?
            .checked_div(10000)
            .ok_or(ContractError::Overflow)?;

        let fee = calc_fee(&env, &seller, amount)?;
        let arbitrator_config = arbitrator.map(ArbitrationConfig::Single);
        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee,
            arbitrator: arbitrator_config,
            status: TradeStatus::Created,
            expiry_time,
            currency: token,
            metadata,
        };
        save_trade(&env, trade_id, &trade);
        events::emit_trade_created(&env, trade_id, seller.clone(), buyer.clone(), amount);
        events::emit_compliance_passed(&env, trade_id, seller, buyer, amount);
        Ok(trade_id)
    }

    /// Create a new trade with multi-signature arbitration for high-value trades.
    /// Requires that all specified arbitrators are registered.
    pub fn create_multisig_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        multisig_config: MultiSigConfig,
        expiry_time: Option<u64>,
        currency: Option<Address>,
        metadata: OptionalMetadata,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }

        // Validate multi-sig configuration
        if multisig_config.arbitrators.len() < multisig_config.threshold as u32 {
            return Err(ContractError::InvalidMultiSigConfig);
        }
        if multisig_config.threshold == 0 {
            return Err(ContractError::InvalidMultiSigConfig);
        }

        // Validate all arbitrators are registered
        for i in 0..multisig_config.arbitrators.len() {
            let arb = multisig_config.arbitrators.get(i).unwrap();
            if !has_arbitrator(&env, &arb) {
                return Err(ContractError::ArbitratorNotRegistered);
            }
        }

        // expiry_time must be in the future (Stellar ledger time is UTC seconds)
        if let Some(expiry) = expiry_time {
            let now = env.ledger().timestamp();
            if expiry <= now {
                return Err(ContractError::InvalidExpiry);
            }
        }

        seller.require_auth();
        validate_user_compliance(&env, &seller, amount)?;
        validate_user_compliance(&env, &buyer, amount)?;

        if let OptionalMetadata::Some(ref meta) = metadata {
            validate_metadata(meta)?;
        }

        // Default to USDC when no currency specified (backward compat)
        let token = currency.unwrap_or(get_usdc_token(&env)?);
        validate_metadata(&metadata)?;

        let trade_id = increment_trade_counter(&env)?;
        let fee = calc_fee(&env, &seller, amount)?;
        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee,
            arbitrator: Some(ArbitrationConfig::MultiSig(multisig_config)),
            status: TradeStatus::Created,
            expiry_time,
            currency: token,
            metadata,
        };
        save_trade(&env, trade_id, &trade);
        events::emit_trade_created(&env, trade_id, seller.clone(), buyer.clone(), amount);
        events::emit_compliance_passed(&env, trade_id, seller, buyer, amount);
        events::emit_trade_created(&env, trade_id, seller, buyer, amount, trade.currency);
        analytics::on_trade_created(&env, amount, &trade.seller, &trade.buyer);
        Ok(trade_id)
    }

    /// Buyer funds the trade
    pub fn fund_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Created {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();
        let token_client = token::Client::new(&env, &trade.currency);
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(
            &trade.buyer,
            &env.current_contract_address(),
            &(trade.amount as i128),
        );
        trade.status = TradeStatus::Funded;
        save_trade(&env, trade_id, &trade);
        events::emit_trade_funded(&env, trade_id);
        analytics::on_trade_funded(&env);
        Ok(())
    pub fn complete_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded {
            return Err(ContractError::InvalidStatus);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Completed;
        save_trade(&env, trade_id, &trade);
        events::emit_trade_completed(&env, trade_id);
        Ok(())
    }

    /// Buyer confirms receipt and releases funds
    pub fn confirm_receipt(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();
        let token_client = token::Client::new(&env, &trade.currency);
        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
        token_client.transfer(
            &env.current_contract_address(),
            &trade.seller,
            &(payout as i128),
        );
        let current_fees = get_currency_fees(&env, &trade.currency);
        let new_fees = current_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?;
        set_currency_fees(&env, &trade.currency, new_fees);
        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &trade.seller, &(payout as i128));
        // Single read-modify-write for fees
        add_accumulated_fees(&env, trade.fee)?;
        tiers::record_volume(&env, &trade.seller, trade.amount)?;
        tiers::record_volume(&env, &trade.buyer, trade.amount)?;
        events::emit_trade_confirmed(&env, trade_id, payout, trade.fee);
        analytics::on_trade_completed(&env, trade.fee);
        Ok(())
    pub fn raise_dispute(env: Env, trade_id: u64, caller: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded && trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        if trade.arbitrator.is_none() {
            return Err(ContractError::ArbitratorNotRegistered);
        }
        // Cannot raise a dispute after the time lock has expired
        if let Some(expiry) = trade.expiry_time {
            if env.ledger().timestamp() >= expiry {
                return Err(ContractError::TradeExpired);
            }
        }
        let caller = env.invoker();
        if caller != trade.buyer && caller != trade.seller {
            return Err(ContractError::Unauthorized);
        }
        caller.require_auth();
        trade.status = TradeStatus::Disputed;

        // Initialize voting start time for multi-sig disputes
        if let Some(ArbitrationConfig::MultiSig(ref mut config)) = trade.arbitrator {
            config.voting_started_at = Some(env.ledger().timestamp());
            // Increment total disputes for each participating arbitrator
            for i in 0..config.arbitrators.len() {
                let arb = config.arbitrators.get(i).unwrap();
                let mut rep = storage::get_arbitrator_reputation(&env, arb);
                rep.total_disputes = rep.total_disputes.saturating_add(1);
                save_arbitrator_reputation(&env, arb, &rep);
            }
        } else if let Some(ArbitrationConfig::Single(ref arb)) = trade.arbitrator {
            let mut rep = storage::get_arbitrator_reputation(&env, arb);
            rep.total_disputes = rep.total_disputes.saturating_add(1);
            save_arbitrator_reputation(&env, arb, &rep);
        }

        save_trade(&env, trade_id, &trade);
        events::emit_dispute_raised(&env, trade_id, caller);
        analytics::on_trade_disputed(&env);
        Ok(())
    /// Use `DisputeResolution::Partial { buyer_bps }` for a split:
    /// `buyer_bps` is the buyer's share of the net payout in basis points (0–10000).
    pub fn resolve_dispute(
        env: Env,
        trade_id: u64,
        resolution: DisputeResolution,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Disputed {
            return Err(ContractError::InvalidStatus);
        }

        match &trade.arbitrator {
            Some(ArbitrationConfig::Single(arbitrator)) => {
                // Single arbitrator resolution - require auth
                arbitrator.require_auth();
                StellarEscrowContract::execute_dispute_resolution(env, trade_id, resolution, trade)?;
            }
            Some(ArbitrationConfig::MultiSig(config)) => {
                // Multi-sig resolution - check consensus
                let summary = StellarEscrowContract::get_voting_summary(env.clone(), trade_id)?;
                if !summary.has_consensus {
                    return Err(ContractError::NoConsensus);
                }
                let consensus_resolution = summary.consensus_resolution.ok_or(ContractError::NoConsensus)?;
                StellarEscrowContract::execute_dispute_resolution(env, trade_id, consensus_resolution, trade)?;
            }
            None => {
                return Err(ContractError::ArbitratorNotRegistered);
            }
        }

        Ok(())
    }

    /// Internal function to execute dispute resolution logic
    fn execute_dispute_resolution(
        env: Env,
        trade_id: u64,
        resolution: DisputeResolution,
        trade: Trade,
    ) -> Result<(), ContractError> {
        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);

        // Net payout after platform fee
        let net = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;

        match resolution {
            DisputeResolution::ReleaseToBuyer => {
                token_client.transfer(&env.current_contract_address(), &trade.buyer, &(net as i128));
                events::emit_dispute_resolved(&env, trade_id, DisputeResolution::ReleaseToBuyer, trade.buyer);
            }
            DisputeResolution::ReleaseToSeller => {
                token_client.transfer(&env.current_contract_address(), &trade.seller, &(net as i128));
                events::emit_dispute_resolved(&env, trade_id, DisputeResolution::ReleaseToSeller, trade.seller);
            }
            DisputeResolution::Partial { buyer_bps } => {
                if buyer_bps > 10000 {
                    return Err(ContractError::InvalidSplitBps);
                }
                // buyer_amount = net * buyer_bps / 10000
                let buyer_amount = net
                    .checked_mul(buyer_bps as u64)
                    .ok_or(ContractError::Overflow)?
                    .checked_div(10000)
                    .ok_or(ContractError::Overflow)?;
                let seller_amount = net.checked_sub(buyer_amount).ok_or(ContractError::Overflow)?;
                if buyer_amount > 0 {
                    token_client.transfer(&env.current_contract_address(), &trade.buyer, &(buyer_amount as i128));
                }
                if seller_amount > 0 {
                    token_client.transfer(&env.current_contract_address(), &trade.seller, &(seller_amount as i128));
                }
                events::emit_partial_resolved(&env, trade_id, buyer_amount, seller_amount, trade.fee);
            }
        }

        // Update accumulated fees
        add_accumulated_fees(&env, trade.fee)?;

        // Update arbitrator reputation stats
        match &trade.arbitrator {
            Some(ArbitrationConfig::Single(arbitrator)) => {
                let mut rep = storage::get_arbitrator_reputation(&env, arbitrator);
                rep.resolved_count = rep.resolved_count.saturating_add(1);
                match resolution {
                    DisputeResolution::ReleaseToBuyer => rep.buyer_wins = rep.buyer_wins.saturating_add(1),
                    DisputeResolution::ReleaseToSeller => rep.seller_wins = rep.seller_wins.saturating_add(1),
                    DisputeResolution::Partial { .. } => {}
                }
                save_arbitrator_reputation(&env, arbitrator, &rep);
                events::emit_arb_rep_updated(&env, arbitrator.clone(), rep.resolved_count, rep.rating_sum, rep.rating_count);
            }
            Some(ArbitrationConfig::MultiSig(config)) => {
                for i in 0..config.arbitrators.len() {
                    let arb = config.arbitrators.get(i).unwrap();
                    let mut rep = storage::get_arbitrator_reputation(&env, arb);
                    rep.resolved_count = rep.resolved_count.saturating_add(1);
                    match resolution {
                        DisputeResolution::ReleaseToBuyer => rep.buyer_wins = rep.buyer_wins.saturating_add(1),
                        DisputeResolution::ReleaseToSeller => rep.seller_wins = rep.seller_wins.saturating_add(1),
                        DisputeResolution::Partial { .. } => {}
                    }
                    save_arbitrator_reputation(&env, arb, &rep);
                    events::emit_arb_rep_updated(&env, arb.clone(), rep.resolved_count, rep.rating_sum, rep.rating_count);
                }
                // Clear votes after resolution
                storage::clear_votes_for_trade(&env, trade_id, &config.arbitrators);
            }
            None => {}
        }
        save_arbitrator_reputation(&env, &arbitrator, &rep);
        events::emit_arb_rep_updated(&env, arbitrator.clone(), rep.resolved_count, rep.rating_sum, rep.rating_count);
        set_currency_fees(&env, &trade.currency, new_fees);
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &recipient, &(payout as i128));
        // Single read-modify-write for fees
        add_accumulated_fees(&env, trade.fee)?;
        let resolution_code: u8 = match resolution {
            DisputeResolution::ReleaseToBuyer => 0,
            DisputeResolution::ReleaseToSeller => 1,
            DisputeResolution::Partial { .. } => 2,
        };
        analytics::on_dispute_resolved(&env, &arbitrator, resolution_code);
        events::emit_dispute_resolved(&env, trade_id, resolution, recipient);
        Ok(()) for the arbitrator of a resolved dispute.
    /// Only the buyer or seller of the trade may rate, once each.
    pub fn rate_arbitrator(env: Env, trade_id: u64, rater: Address, stars: u32) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        if stars < 1 || stars > 5 {
            return Err(ContractError::InvalidRating);
        }
        rater.require_auth();
        let trade = get_trade(&env, trade_id)?;
        // Only buyer or seller may rate
        if rater != trade.buyer && rater != trade.seller {
            return Err(ContractError::Unauthorized);
        }
        // Trade must have been disputed (and resolved) — status check
        if trade.status != TradeStatus::Disputed {
            return Err(ContractError::InvalidStatus);
        }
        let arbitrator = trade.arbitrator.ok_or(ContractError::NoArbitrator)?;
        if has_rated(&env, trade_id, &rater) {
            return Err(ContractError::AlreadyRated);
        }
        mark_rated(&env, trade_id, &rater);
        let mut rep = storage::get_arbitrator_reputation(&env, &arbitrator);
        rep.rating_sum = rep.rating_sum.saturating_add(stars);
        rep.rating_count = rep.rating_count.saturating_add(1);
        save_arbitrator_reputation(&env, &arbitrator, &rep);
        events::emit_arb_rated(&env, arbitrator.clone(), trade_id, rater, stars);
        events::emit_arb_rep_updated(&env, arbitrator, rep.resolved_count, rep.rating_sum, rep.rating_count);
        Ok(())
    }

    /// Query reputation stats for an arbitrator.
    pub fn get_arbitrator_reputation(env: Env, arbitrator: Address) -> ArbitratorReputation {
        storage::get_arbitrator_reputation(&env, &arbitrator)
    }

    /// Cancel an unfunded trade
    pub fn cancel_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Created {
            return Err(ContractError::InvalidStatus);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Cancelled;
        save_trade(&env, trade_id, &trade);
        events::emit_trade_cancelled(&env, trade_id);
        analytics::on_trade_cancelled(&env);
        Ok(()): anyone can call this once the expiry has
    /// passed and the trade is Funded or Completed (not Disputed/Cancelled).
    /// Funds are released to the seller minus the platform fee.
    pub fn claim_time_release(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        let trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded && trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        let expiry = trade.expiry_time.ok_or(ContractError::InvalidExpiry)?;
        // Stellar ledger timestamp is always UTC seconds — no timezone handling needed
        if env.ledger().timestamp() < expiry {
            return Err(ContractError::TradeNotExpired);
        }
        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);
        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
        token_client.transfer(&env.current_contract_address(), &trade.seller, &(payout as i128));
        let current_fees = get_accumulated_fees(&env)?;
        let new_fees = current_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?;
        set_accumulated_fees(&env, new_fees);
        tiers::record_volume(&env, &trade.seller, trade.amount)?;
        tiers::record_volume(&env, &trade.buyer, trade.amount)?;
        events::emit_time_released(&env, trade_id, trade.seller, payout);
        Ok(())
    }

    /// Get trade details
    pub fn get_trade(env: Env, trade_id: u64) -> Result<Trade, ContractError> {
        get_trade(&env, trade_id)
    }

    /// Get accumulated fees (USDC only — backward-compatible)
    pub fn get_accumulated_fees(env: Env) -> Result<u64, ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let usdc = get_usdc_token(&env)?;
        Ok(get_currency_fees(&env, &usdc))
    }

    /// Get accumulated fees for a specific currency
    pub fn get_fees_for_currency(env: Env, currency: Address) -> u64 {
        get_currency_fees(&env, &currency)
    }

    /// Check if arbitrator is registered
    pub fn is_arbitrator_registered(env: Env, arbitrator: Address) -> bool {
        has_arbitrator(&env, &arbitrator)
    }

    /// Get platform fee in basis points
    pub fn get_platform_fee_bps(env: Env) -> Result<u32, ContractError> {
        get_fee_bps(&env)
    }

    // -------------------------------------------------------------------------
    // Advanced Query Functions
    // -------------------------------------------------------------------------

    /// Filter, sort, and paginate trades.
    ///
    /// - `filter`: optional criteria (status, participant, amount range, id range)
    /// - `page`: pagination + sort options (offset, limit ≤ 100, sort_by, direction)
    pub fn query_trades(
        env: Env,
        filter: queries::TradeFilter,
        page: queries::PageParams,
    ) -> Result<soroban_sdk::Vec<Trade>, ContractError> {
        require_initialized(&env)?;
        queries::query_trades(&env, filter, page)
    }

    /// Aggregate statistics (count, volume, fees, min/max amount) over filtered trades.
    pub fn aggregate_trades(
        env: Env,
        filter: queries::TradeFilter,
    ) -> Result<queries::TradeStats, ContractError> {
        require_initialized(&env)?;
        queries::aggregate_trades(&env, filter)
    }

    // -------------------------------------------------------------------------
    // Oracle Integration
    // -------------------------------------------------------------------------

    /// Register a price oracle contract for a `base`/`quote` asset pair (admin only).
    /// `priority`: lower = queried first. Up to 5 oracles per pair.
    pub fn register_oracle(
        env: Env,
        base: Address,
        quote: Address,
        oracle: Address,
        priority: u32,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        get_admin(&env)?.require_auth();
        oracle::register_oracle(&env, &base, &quote, oracle, priority)
    }

    /// Remove a price oracle for a `base`/`quote` pair (admin only).
    pub fn remove_oracle(
        env: Env,
        base: Address,
        quote: Address,
        oracle: Address,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        get_admin(&env)?.require_auth();
        oracle::remove_oracle(&env, &base, &quote, &oracle)
    }

    /// List all registered oracles for a `base`/`quote` pair.
    pub fn get_oracles(
        env: Env,
        base: Address,
        quote: Address,
    ) -> soroban_sdk::Vec<oracle::OracleEntry> {
        oracle::get_oracles(&env, &base, &quote)
    }

    /// Fetch the current price for `base`/`quote` from registered oracles.
    /// Queries in priority order; returns first fresh (non-stale) response.
    /// Returns `Err(OracleUnavailable)` if all sources fail or are stale.
    pub fn get_oracle_price(
        env: Env,
        base: Address,
        quote: Address,
    ) -> Result<oracle::PriceData, ContractError> {
        oracle::get_price(&env, &base, &quote)
    }

    /// Validate that `trade_amount` falls within `[min_usd, max_usd]` at the
    /// current oracle price. Bounds are in oracle-scaled units (value × 10^decimals).
    /// Returns `Err(OracleUnavailable)` on oracle failure — caller decides whether to block.
    pub fn validate_trade_price(
        env: Env,
        base: Address,
        quote: Address,
        trade_amount: u64,
        min_usd: i128,
        max_usd: i128,
    ) -> Result<oracle::PriceValidation, ContractError> {
        oracle::validate_trade_price(&env, &base, &quote, trade_amount, min_usd, max_usd)
    }

    // -------------------------------------------------------------------------
    // Emergency Pause
    // -------------------------------------------------------------------------

    /// Pause all contract operations (admin only).
    pub fn pause(env: Env) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_paused(&env, true);
        events::emit_paused(&env, admin);
        Ok(())
    }

    /// Unpause the contract (admin only).
    pub fn unpause(env: Env) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_paused(&env, false);
        events::emit_unpaused(&env, admin);
        Ok(())
    }

    /// Emergency withdrawal of all contract token balance (admin only).
    pub fn emergency_withdraw(env: Env, to: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);
    /// Allowed even while paused so funds can always be recovered.
    pub fn emergency_withdraw(env: Env, to: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        let balance = token_client.balance(&env.current_contract_address());
        if balance > 0 {
            token_client.transfer(&env.current_contract_address(), &to, &balance);
        }
        set_accumulated_fees(&env, 0);
        events::emit_emergency_withdraw(&env, to, balance as u64);
        Ok(())
    }

    /// Returns true if the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        is_paused(&env)
    }

    // -------------------------------------------------------------------------
    // Metadata
    // -------------------------------------------------------------------------

    /// Update or replace metadata on an existing trade (seller only)
    pub fn update_trade_metadata(
        env: Env,
        trade_id: u64,
        metadata: OptionalMetadata,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        trade.seller.require_auth();
        validate_metadata(&metadata)?;
        trade.metadata = metadata;
        save_trade(&env, trade_id, &trade);
        events::emit_metadata_updated(&env, trade_id);
        Ok(())
    }

    /// Get metadata for a trade
    pub fn get_trade_metadata(env: Env, trade_id: u64) -> Result<OptionalMetadata, ContractError> {
        Ok(get_trade(&env, trade_id)?.metadata)
    }

    // -------------------------------------------------------------------------
    // Fee Tier System
    // -------------------------------------------------------------------------

    /// Admin: configure fee rates per tier.
    pub fn set_tier_config(env: Env, config: TierConfig) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        tiers::set_tier_config(&env, &config)
    }

    /// Admin: assign a custom fee rate to a specific user.
    pub fn set_user_custom_fee(env: Env, user: Address, fee_bps: u32) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        tiers::set_custom_fee(&env, &user, fee_bps)
    }

    /// Admin: remove a user's custom fee, reverting to volume-based tier.
    pub fn remove_user_custom_fee(env: Env, user: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        tiers::remove_custom_fee(&env, &user);
        Ok(())
    }

    /// Query a user's current tier info.
    pub fn get_user_tier(env: Env, user: Address) -> Option<UserTierInfo> {
        storage::get_user_tier(&env, &user)
    }

    /// Query the current tier fee configuration.
    pub fn get_tier_config(env: Env) -> Option<TierConfig> {
        storage::get_tier_config(&env)
    }

    /// Query the effective fee bps for a user's next trade.
    pub fn get_effective_fee_bps(env: Env, user: Address) -> Result<u32, ContractError> {
        let base = get_fee_bps(&env)?;
        Ok(tiers::effective_fee_bps(&env, &user, base))
    }

    // -------------------------------------------------------------------------
    // Trade Templates
    // -------------------------------------------------------------------------

    /// Create a reusable trade template (owner = seller).
    pub fn create_template(
        env: Env,
        owner: Address,
        name: soroban_sdk::String,
        terms: TemplateTerms,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        owner.require_auth();
        templates::create_template(&env, &owner, name, terms)
    }

    /// Update a template with new terms, bumping its version.
    pub fn update_template(
        env: Env,
        caller: Address,
        template_id: u64,
        name: soroban_sdk::String,
        terms: TemplateTerms,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        caller.require_auth();
        templates::update_template(&env, &caller, template_id, name, terms)
    }

    /// Deactivate a template so it can no longer be used to create trades.
    pub fn deactivate_template(
        env: Env,
        caller: Address,
        template_id: u64,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        caller.require_auth();
        templates::deactivate_template(&env, &caller, template_id)
    }

    /// Create a trade from a template.
    pub fn create_trade_from_template(
        env: Env,
        seller: Address,
        buyer: Address,
        template_id: u64,
        amount: u64,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        seller.require_auth();

        let (terms, version) = templates::resolve_terms(&env, template_id)?;

        if let Some(fixed) = terms.fixed_amount {
            if amount != fixed {
                return Err(ContractError::TemplateAmountMismatch);
            }
        }
        if let Some(ref arb) = terms.default_arbitrator {
            if !has_arbitrator(&env, arb) {
                return Err(ContractError::ArbitratorNotRegistered);
            }
        }

        let trade_id = increment_trade_counter(&env)?;
        let base_fee_bps = get_fee_bps(&env)?;
        let effective_bps = tiers::effective_fee_bps(&env, &seller, base_fee_bps);
        let discount = subscription::subscription_discount_bps(&env, &seller);
        let final_bps = effective_bps.saturating_sub(discount);
        let fee = amount
            .checked_mul(final_bps as u64)
            .ok_or(ContractError::Overflow)?
            .checked_div(10000)
            .ok_or(ContractError::Overflow)?;
        let fee = calc_fee(&env, &seller, amount)?;

        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee,
            arbitrator: terms.default_arbitrator.map(ArbitrationConfig::Single),
            status: TradeStatus::Created,
            expiry_time: None,
            currency: get_usdc_token(&env)?,
            metadata: terms.default_metadata,
        };

        save_trade(&env, trade_id, &trade);
        events::emit_trade_created(&env, trade_id, seller, buyer, amount);
        events::emit_template_trade(&env, trade_id, template_id, version);
        Ok(trade_id)
    }

    /// Get a template by ID.
    pub fn get_template(env: Env, template_id: u64) -> Result<TradeTemplate, ContractError> {
        storage::get_template(&env, template_id)
    }

    // -------------------------------------------------------------------------
    // Subscription Model
    // -------------------------------------------------------------------------

    /// Purchase a new subscription. Payment (USDC) is transferred to the admin.
    pub fn subscribe(
        env: Env,
        subscriber: Address,
        tier: SubscriptionTier,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        subscriber.require_auth();
        let admin = get_admin(&env)?;
        subscription::subscribe(&env, &subscriber, tier, &admin)
    }

    /// Renew an existing subscription for another period.
    pub fn renew_subscription(env: Env, subscriber: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        subscriber.require_auth();
        let admin = get_admin(&env)?;
        subscription::renew(&env, &subscriber, &admin)
    }

    /// Cancel a subscription immediately (no refund).
    pub fn cancel_subscription(env: Env, subscriber: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        subscriber.require_auth();
        subscription::cancel(&env, &subscriber)
    }

    /// Get subscription details for a user.
    pub fn get_subscription(env: Env, subscriber: Address) -> Option<Subscription> {
        subscription::get(&env, &subscriber)
    }

    // -------------------------------------------------------------------------
    // Governance
    // -------------------------------------------------------------------------

    /// Admin: set the governance token address (one-time setup).
    pub fn set_gov_token(env: Env, token: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        storage::set_gov_token(&env, &token);
        Ok(())
    }

    /// Create a governance proposal.
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        action: ProposalAction,
    ) -> Result<u64, ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        proposer.require_auth();
        governance::create_proposal(&env, &proposer, action)
    }

    /// Vote on a proposal.
    pub fn cast_vote(
        env: Env,
        voter: Address,
        proposal_id: u64,
        support: bool,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        voter.require_auth();
        governance::cast_vote(&env, &voter, proposal_id, support)
    }

    /// Execute a passed proposal after voting ends.
    pub fn execute_proposal(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        governance::execute_proposal(&env, proposal_id)
    }

    /// Delegate voting power to another address.
    pub fn delegate(env: Env, delegator: Address, delegatee: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        delegator.require_auth();
        governance::delegate(&env, &delegator, &delegatee);
        Ok(())
    }

    /// Remove delegation, reclaiming own voting power.
    pub fn undelegate(env: Env, delegator: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        delegator.require_auth();
        governance::undelegate(&env, &delegator);
        Ok(())
    }

    /// Get a proposal by ID.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, ContractError> {
        governance::get(&env, proposal_id)
    }

    /// Get total number of proposals created.
    pub fn get_proposal_count(env: Env) -> u64 {
        governance::proposal_count(&env)
    // Upgrade Mechanism
    // -------------------------------------------------------------------------

    /// Step 1: Admin proposes a WASM upgrade with a 24-hour timelock.
    /// Replaces any existing pending proposal. Emits `EvUpgradeProposed`.
    pub fn propose_upgrade(
        env: Env,
        admin: Address,
        new_wasm_hash: BytesN<32>,
        description: soroban_sdk::String,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        upgrade::propose_upgrade(&env, &admin, new_wasm_hash, description)
    }

    /// Cancel a pending upgrade proposal before it is executed.
    pub fn cancel_upgrade(env: Env, admin: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        upgrade::cancel_upgrade(&env, &admin)
    }

    /// Step 2: Execute the upgrade after the timelock has passed.
    /// Snapshots state for rollback, swaps WASM, sets upgrade guard.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        upgrade::execute_upgrade(&env, &admin)
    }

    /// Step 3: Run post-upgrade state migrations and finalise the upgrade.
    /// `expected_version` must match the current stored version.
    pub fn migrate(env: Env, expected_version: u32) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        upgrade::run_migration(&env, &admin, expected_version)
    }

    /// Rollback the upgrade within the 1-hour rollback window.
    /// Restores version and fee bps from the pre-upgrade snapshot.
    pub fn rollback_upgrade(env: Env, admin: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        upgrade::rollback_upgrade(&env, &admin)
    }

    /// Returns the current contract version.
    pub fn version(env: Env) -> u32 {
        get_version(&env)
    }

    /// Returns the pending upgrade proposal, if any.
    pub fn get_upgrade_proposal(env: Env) -> Option<upgrade::UpgradeProposal> {
        upgrade::get_upgrade_proposal(&env)
    }

    /// Returns the rollback snapshot, if an upgrade is in progress.
    pub fn get_rollback_snapshot(env: Env) -> Option<upgrade::RollbackSnapshot> {
        upgrade::get_rollback_snapshot(&env)
    }

    // -------------------------------------------------------------------------
    // Cross-Chain Bridge Support
    // -------------------------------------------------------------------------

    /// Set the trusted bridge oracle address (admin only).
    /// The oracle is an off-chain relayer that submits deposit confirmations.
    pub fn set_bridge_oracle(env: Env, oracle: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_bridge_oracle(&env, &oracle);
        events::emit_bridge_oracle_set(&env, oracle);
        Ok(())
    }

    /// Create a cross-chain trade. Funds arrive via bridge; status starts as AwaitingBridge.
    /// `expiry_ledgers`: how many ledgers from now before the trade can be expired.
    pub fn create_cross_chain_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        arbitrator: Option<Address>,
        source_chain: soroban_sdk::String,
        expiry_ledgers: u32,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        get_bridge_oracle(&env).ok_or(ContractError::BridgeOracleNotSet)?;
        seller.require_auth();
        if let Some(ref arb) = arbitrator {
            if !has_arbitrator(&env, arb) {
                return Err(ContractError::ArbitratorNotRegistered);
            }
        }
        let trade_id = increment_trade_counter(&env)?;
        let fee = calc_fee(&env, &seller, amount)?;
        let expires_at_ledger = env.ledger().sequence()
            .checked_add(expiry_ledgers)
            .ok_or(ContractError::Overflow)?;

        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee,
            arbitrator: arbitrator.map(ArbitrationConfig::Single),
            status: TradeStatus::AwaitingBridge,
            expiry_time: None,
            currency: get_usdc_token(&env)?,
            metadata: OptionalMetadata::None,
        };
        save_trade(&env, trade_id, &trade);
        save_cross_chain_info(&env, trade_id, &CrossChainInfo {
            source_chain: source_chain.clone(),
            source_tx_hash: soroban_sdk::String::from_str(&env, ""),
            expires_at_ledger,
        });
        events::emit_bridge_trade_created(&env, trade_id, source_chain);
        Ok(trade_id)
    }

    /// Called by the bridge oracle to confirm a deposit arrived from the source chain.
    /// Transitions the trade from AwaitingBridge → Funded.
    pub fn confirm_bridge_deposit(
        env: Env,
        trade_id: u64,
        source_tx_hash: soroban_sdk::String,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let oracle = get_bridge_oracle(&env).ok_or(ContractError::BridgeOracleNotSet)?;
        oracle.require_auth();

        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::AwaitingBridge {
            return Err(ContractError::InvalidStatus);
        }
        let mut info = get_cross_chain_info(&env, trade_id)
            .ok_or(ContractError::TradeNotFound)?;
        if env.ledger().sequence() > info.expires_at_ledger {
            return Err(ContractError::BridgeTradeExpired);
        }
        // Record the source tx hash for auditability
        info.source_tx_hash = source_tx_hash;
        save_cross_chain_info(&env, trade_id, &info);

        trade.status = TradeStatus::Funded;
        save_trade(&env, trade_id, &trade);
        events::emit_bridge_deposit_confirmed(&env, trade_id);
        Ok(())
    }

    /// Expire a cross-chain trade that was never confirmed by the oracle.
    /// Callable by the seller after the expiry ledger has passed.
    pub fn expire_bridge_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::AwaitingBridge {
            return Err(ContractError::InvalidStatus);
        }
        let info = get_cross_chain_info(&env, trade_id)
            .ok_or(ContractError::TradeNotFound)?;
        if env.ledger().sequence() <= info.expires_at_ledger {
            return Err(ContractError::BridgeTradeNotExpired);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Cancelled;
        save_trade(&env, trade_id, &trade);
        events::emit_bridge_trade_expired(&env, trade_id);
        Ok(())
    }

    /// Get cross-chain info for a trade.
    pub fn get_cross_chain_info(env: Env, trade_id: u64) -> Option<CrossChainInfo> {
        get_cross_chain_info(&env, trade_id)
    }

    // -------------------------------------------------------------------------
    // Trade Insurance
    // -------------------------------------------------------------------------

    /// Register an insurance provider (admin only).
    pub fn register_insurance_provider(env: Env, provider: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        save_insurance_provider(&env, &provider);
        events::emit_insurance_provider_registered(&env, provider);
        Ok(())
    }

    /// Remove an insurance provider (admin only).
    pub fn remove_insurance_provider(env: Env, provider: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        remove_insurance_provider(&env, &provider);
        events::emit_insurance_provider_removed(&env, provider);
        Ok(())
    }

    /// Purchase insurance for a funded trade.
    /// The buyer pays the premium now; it is transferred to the provider immediately.
    /// `premium_bps`: premium as basis points of trade amount (max 1000 = 10%).
    /// `coverage`: maximum additional payout the provider guarantees.
    pub fn purchase_insurance(
        env: Env,
        trade_id: u64,
        provider: Address,
        premium_bps: u32,
        coverage: u64,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if premium_bps > types::MAX_INSURANCE_PREMIUM_BPS {
            return Err(ContractError::InsurancePremiumTooHigh);
        }
        if !has_insurance_provider(&env, &provider) {
            return Err(ContractError::InsuranceProviderNotRegistered);
        }
        let trade = get_trade(&env, trade_id)?;
        // Insurance can be purchased on a Funded or Completed trade (not yet settled)
        if trade.status != TradeStatus::Funded && trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();

        let premium = trade.amount
            .checked_mul(premium_bps as u64).ok_or(ContractError::Overflow)?
            .checked_div(10000).ok_or(ContractError::Overflow)?;

        // Transfer premium from buyer to provider
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&trade.buyer, &provider, &(premium as i128));

        save_insurance_policy(&env, trade_id, &InsurancePolicy {
            provider: provider.clone(),
            premium,
            coverage,
            claimed: false,
        });
        events::emit_insurance_purchased(&env, trade_id, provider, premium, coverage);
        Ok(())
    }

    /// File an insurance claim on a disputed trade (provider pays out).
    /// Only callable by the registered provider for this trade's policy.
    /// `recipient`: buyer or seller — whoever the provider decides to compensate.
    /// The provider transfers `payout` (up to `coverage`) directly to the recipient.
    pub fn claim_insurance(
        env: Env,
        trade_id: u64,
        recipient: Address,
        payout: u64,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let trade = get_trade(&env, trade_id)?;
        // Claims only valid on disputed or completed trades
        if trade.status != TradeStatus::Disputed && trade.status != TradeStatus::Completed {
            return Err(ContractError::InsuranceClaimNotEligible);
        }
        let mut policy = get_insurance_policy(&env, trade_id)
            .ok_or(ContractError::TradeNotInsured)?;
        if policy.claimed {
            return Err(ContractError::InsuranceAlreadyClaimed);
        }
        // Payout capped at coverage
        let actual_payout = payout.min(policy.coverage);
        policy.provider.require_auth();

        // Provider transfers payout to recipient from their own balance
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&policy.provider, &recipient, &(actual_payout as i128));

        policy.claimed = true;
        save_insurance_policy(&env, trade_id, &policy);
        events::emit_insurance_claimed(&env, trade_id, actual_payout, recipient);
        Ok(())
    }

    /// Check if a provider is registered.
    pub fn is_insurance_provider_registered(env: Env, provider: Address) -> bool {
        has_insurance_provider(&env, &provider)
    }

    /// Get the insurance policy for a trade, if any.
    pub fn get_insurance_policy(env: Env, trade_id: u64) -> Option<InsurancePolicy> {
        get_insurance_policy(&env, trade_id)
    }

    // -------------------------------------------------------------------------
    // Privacy Features
    // -------------------------------------------------------------------------

    /// Set privacy settings for a trade (seller or buyer only).
    pub fn set_trade_privacy(
        env: Env,
        caller: Address,
        trade_id: u64,
        data_hash: soroban_sdk::String,
        encrypted_ptr: Option<soroban_sdk::String>,
        private_arbitration: bool,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        caller.require_auth();
        privacy::set_trade_privacy(&env, &caller, trade_id, data_hash, encrypted_ptr, private_arbitration)
    }

    /// Grant selective disclosure to a third party.
    pub fn grant_disclosure(
        env: Env,
        caller: Address,
        trade_id: u64,
        grantee: Address,
        encrypted_key: soroban_sdk::String,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        caller.require_auth();
        privacy::grant_disclosure(&env, &caller, trade_id, grantee, encrypted_key)
    }

    /// Revoke a disclosure grant.
    pub fn revoke_disclosure(
        env: Env,
        caller: Address,
        trade_id: u64,
        grantee: Address,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        caller.require_auth();
        privacy::revoke_disclosure(&env, &caller, trade_id, grantee)
    }

    /// Get privacy settings for a trade.
    pub fn get_trade_privacy(env: Env, trade_id: u64) -> Option<TradePrivacy> {
        privacy::get_privacy(&env, trade_id)
    }

    /// Get a disclosure grant for a specific grantee.
    pub fn get_disclosure_grant(
        env: Env,
        trade_id: u64,
        grantee: Address,
    ) -> Result<DisclosureGrant, ContractError> {
        privacy::get_grant(&env, trade_id, &grantee)
    }

    // -----------------------------------------------------------------------
    // Analytics
    // -----------------------------------------------------------------------

    /// Return raw on-chain platform metrics (volume, trade counts, fees).
    pub fn get_platform_metrics(env: Env) -> analytics::PlatformMetrics {
        analytics::get_metrics(&env)
    }

    /// Return derived platform statistics: success rate, dispute rate, active trades.
    pub fn get_platform_stats(env: Env) -> analytics::PlatformStats {
        analytics::get_stats(&env)
    }

    /// Return performance metrics for a specific arbitrator.
    pub fn get_arbitrator_analytics(env: Env, arbitrator: Address) -> analytics::ArbitratorMetrics {
        analytics::get_arb_metrics(&env, &arbitrator)
    }

    /// Unified analytics query: returns all-time stats, a time-windowed snapshot,
    /// and the count of unique addresses that have interacted with the contract.
    ///
    /// `window` selects the time period: `Last24h`, `Last7d`, `Last30d`, or `AllTime`.
    pub fn analytics_query(env: Env, window: analytics::TimeWindow) -> analytics::AnalyticsResult {
        analytics::analytics_query(&env, window)
    }

    // AMM — Automated Market Making
    // -----------------------------------------------------------------------

    /// Create a new constant-product liquidity pool for a token pair.
    /// `fee_bps` is the swap fee in basis points (e.g. 30 = 0.30 %).
    /// Returns the new pool id.
    pub fn amm_create_pool(
        env: Env,
        token_a: Address,
        token_b: Address,
        fee_bps: u32,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        amm::create_pool(&env, token_a, token_b, fee_bps)
    }

    /// Add liquidity to a pool.
    /// `min_shares` enforces slippage protection on the minted LP shares.
    /// Returns the number of LP shares minted.
    pub fn amm_add_liquidity(
        env: Env,
        pool_id: u64,
        provider: Address,
        amount_a: u64,
        amount_b: u64,
        min_shares: u64,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        amm::add_liquidity(&env, pool_id, &provider, amount_a, amount_b, min_shares)
    }

    /// Remove liquidity from a pool.
    /// `min_a` / `min_b` enforce slippage protection on the returned amounts.
    /// Returns `(amount_a, amount_b)` transferred back to the provider.
    pub fn amm_remove_liquidity(
        env: Env,
        pool_id: u64,
        provider: Address,
        shares: u64,
        min_a: u64,
        min_b: u64,
    ) -> Result<(u64, u64), ContractError> {
        require_initialized(&env)?;
        amm::remove_liquidity(&env, pool_id, &provider, shares, min_a, min_b)
    }

    /// Swap `amount_in` of `token_in` for the other token in the pool.
    /// `min_out` enforces slippage protection — reverts if output < min_out.
    pub fn amm_swap(
        env: Env,
        pool_id: u64,
        caller: Address,
        token_in: Address,
        amount_in: u64,
        min_out: u64,
    ) -> Result<amm::SwapResult, ContractError> {
        require_initialized(&env)?;
        amm::swap(&env, pool_id, &caller, &token_in, amount_in, min_out)
    }

    /// Simulate a swap without executing it. Returns expected output and price impact.
    pub fn amm_quote_swap(
        env: Env,
        pool_id: u64,
        token_in: Address,
        amount_in: u64,
    ) -> Result<amm::SwapResult, ContractError> {
        amm::quote_swap(&env, pool_id, &token_in, amount_in)
    }

    /// Get the current spot price of token_a in terms of token_b (scaled by 1e7).
    pub fn amm_spot_price(env: Env, pool_id: u64) -> Result<u64, ContractError> {
        amm::spot_price(&env, pool_id)
    }

    /// Claim accumulated swap-fee yield for a liquidity provider.
    /// Returns `(claimed_a, claimed_b)`.
    pub fn amm_claim_yield(
        env: Env,
        pool_id: u64,
        provider: Address,
    ) -> Result<(u64, u64), ContractError> {
        require_initialized(&env)?;
        amm::claim_yield(&env, pool_id, &provider)
    }

    /// Get pool state.
    pub fn amm_get_pool(env: Env, pool_id: u64) -> Result<amm::Pool, ContractError> {
        amm::get_pool(&env, pool_id)
    }

    /// Get LP position for a provider in a pool.
    pub fn amm_get_lp_position(env: Env, pool_id: u64, provider: Address) -> amm::LpPosition {
        amm::get_lp_position(&env, pool_id, &provider)
    }
}


