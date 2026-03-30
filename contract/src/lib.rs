#![no_std]

#[cfg(test)]
extern crate std;

mod analytics;
mod errors;
mod events;
mod storage;
pub mod types;
mod subscription;
mod templates;
mod tiers;
mod types;
mod upgrade;
mod proxy;
mod insurance;

use soroban_sdk::{contract, contractimpl, token, Address, Env, String};

pub use analytics::{
    AnalyticsResult, ArbitratorMetrics, PeriodAnalytics, PlatformMetrics, PlatformStats,
    SuccessRateStats, TimeWindow, VolumeStats,
};
pub use errors::ContractError;
pub use types::{
    ArbitrationConfig, CrossChainInfo, DisputeResolution, InsurancePolicy, KycStatus,
    OptionalMetadata, Trade, TradeStatus, UserCompliance, MAX_INSURANCE_PREMIUM_BPS,
    MAX_METADATA_SIZE,
    ArbitrationConfig, ArbitratorReputation, ArbitratorVote, DisclosureGrant, DisputeResolution,
    MultiSigConfig, PriceTrigger, Proposal, ProposalAction, ProposalStatus, Subscription,
    SubscriptionTier, TemplateTerms, TemplateVersion, Trade, TradePrivacy, TradeStatus,
    TradeTemplate, TriggerAction, UserTier, UserTierInfo, VotingSummary,
};
pub use queries::{PageParams, SortDirection, TradeFilter, TradeSortField, TradeStats};
pub use oracle::{OracleEntry, PriceData, PriceValidation};
pub use bridge::{BridgeProvider, CrossChainTrade, BridgeAttestation, BridgeValidation};
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
    has_insurance_provider, save_insurance_provider, remove_insurance_provider,
};

fn require_initialized(env: &Env) -> Result<(), ContractError> {
    if !storage::is_initialized(env) {
        return Err(ContractError::NotInitialized);
    }
    Ok(())
}

fn require_not_paused(env: &Env) -> Result<(), ContractError> {
    if storage::is_paused(env) {
        return Err(ContractError::ContractPaused);
    }
    Ok(())
}

fn validate_metadata(metadata: &OptionalMetadata) -> Result<(), ContractError> {
    match metadata {
        OptionalMetadata::None => Ok(()),
        OptionalMetadata::Some(value) => {
            let len = value.len();
            if len == 0 {
                Err(ContractError::InvalidMetadata)
            } else if len > MAX_METADATA_SIZE {
                Err(ContractError::MetadataValueTooLong)
            } else {
                Ok(())
            }
        }
    }
}

fn validate_user_compliance(env: &Env, user: &Address, amount: u64) -> Result<(), ContractError> {
    let compliance = storage::get_user_compliance(env, user).ok_or(ContractError::ComplianceDataMissing)?;
    if compliance.kyc_status != KycStatus::Verified {
        return Err(ContractError::KycNotVerified);
    }
    if !compliance.aml_cleared {
        return Err(ContractError::AmlNotCleared);
    }
    if !storage::is_jurisdiction_allowed(env, &compliance.jurisdiction) {
        return Err(ContractError::JurisdictionRestricted);
    }
    let user_limit = storage::get_user_trade_limit(env, user);
    if user_limit > 0 && amount > user_limit {
        return Err(ContractError::TradeAmountLimitExceeded);
    }
    if amount > storage::get_global_trade_limit(env) {
        return Err(ContractError::TradeAmountLimitExceeded);
    }
    Ok(())
}

fn calc_fee(env: &Env, amount: u64) -> Result<u64, ContractError> {
    amount
        .checked_mul(storage::get_fee_bps(env)? as u64)
        .ok_or(ContractError::Overflow)?
        .checked_div(10_000)
        .ok_or(ContractError::Overflow)
}

fn usdc_client<'a>(env: &'a Env) -> Result<token::Client<'a>, ContractError> {
    Ok(token::Client::new(env, &storage::get_usdc_token(env)?))
}

#[contract]
pub struct StellarEscrowContract;

#[contractimpl]
impl StellarEscrowContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        usdc_token: Address,
        fee_bps: u32,
    ) -> Result<(), ContractError> {
        if storage::is_initialized(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        if fee_bps > 10_000 {
            return Err(ContractError::InvalidFeeBps);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_usdc_token(&env, &usdc_token);
        storage::set_fee_bps(&env, fee_bps);
        storage::set_trade_counter(&env, 0);
        storage::set_accumulated_fees(&env, 0);
        storage::set_version(&env, 1);
        storage::set_initialized(&env);
        Ok(())
    }

    pub fn register_arbitrator(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        storage::get_admin(&env)?.require_auth();
        storage::save_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_registered(&env, arbitrator);
        Ok(())
    }

    pub fn remove_arbitrator_fn(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        storage::get_admin(&env)?.require_auth();
        storage::remove_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_removed(&env, arbitrator);
        Ok(())
    }

    pub fn is_arbitrator_registered(env: Env, arbitrator: Address) -> bool {
        storage::has_arbitrator(&env, &arbitrator)
    // -------------------------------------------------------------------------
    // Multi-Signature Arbitration
    // -------------------------------------------------------------------------

    /// Create a trade with multi-signature arbitration.
    /// All arbitrators in `config` must be registered; threshold must be > 0
    /// and ≤ arbitrators count.

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
    }

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

    pub fn update_fee(env: Env, fee_bps: u32) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if fee_bps > 10_000 {
            return Err(ContractError::InvalidFeeBps);
        }
        storage::get_admin(&env)?.require_auth();
        storage::set_fee_bps(&env, fee_bps);
        events::emit_fee_updated(&env, fee_bps);
        Ok(())
    }

    pub fn get_platform_fee_bps(env: Env) -> Result<u32, ContractError> {
        storage::get_fee_bps(&env)
    // -------------------------------------------------------------------------
    // Trade Insurance
    // -------------------------------------------------------------------------

    /// Register an insurance provider (admin only)
    pub fn register_insurance_provider(env: Env, provider: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        save_insurance_provider(&env, &provider);
        events::emit_insurance_provider_registered(&env, provider);
        Ok(())
    }

    /// Remove an insurance provider (admin only)
    pub fn remove_insurance_provider_fn(env: Env, provider: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        remove_insurance_provider(&env, &provider);
        events::emit_insurance_provider_removed(&env, provider);
        Ok(())
    }

    /// Purchase optional trade insurance for a created trade
    pub fn purchase_insurance(
        env: Env,
        trade_id: u64,
        buyer: Address,
        provider: Address,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        insurance::purchase_insurance(&env, trade_id, buyer, provider)
    }

    /// Claim insurance payout for a disputed trade
    pub fn claim_insurance(
        env: Env,
        trade_id: u64,
        recipient: Address,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        insurance::claim_insurance(&env, trade_id, recipient)
    }

    /// Calculate insurance premium for a given trade amount
    pub fn get_insurance_premium(env: Env, amount: u64, provider: Address) -> u64 {
        insurance::calculate_premium(&env, amount, &provider)
    }

    pub fn set_user_compliance(
        env: Env,
        admin: Address,
        user: Address,
        compliance: UserCompliance,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if admin != storage::get_admin(&env)? {
            return Err(ContractError::Unauthorized);
        }
        admin.require_auth();
        storage::save_user_compliance(&env, &user, &compliance);
        events::emit_compliance_updated(&env, user);
        Ok(())
    }

    pub fn get_user_compliance(env: Env, user: Address) -> Option<UserCompliance> {
        storage::get_user_compliance(&env, &user)
    }

    pub fn set_user_trade_limit(
        env: Env,
        admin: Address,
        user: Address,
        limit: u64,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if admin != storage::get_admin(&env)? {
            return Err(ContractError::Unauthorized);
        }
        admin.require_auth();
        storage::set_user_trade_limit(&env, &user, limit);
        Ok(())
    }

    pub fn set_jurisdiction_rule(
        env: Env,
        admin: Address,
        jurisdiction: String,
        allowed: bool,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if admin != storage::get_admin(&env)? {
            return Err(ContractError::Unauthorized);
        }
        admin.require_auth();
        storage::set_jurisdiction_rule(&env, &jurisdiction, allowed);
        Ok(())
    }

    pub fn set_global_trade_limit(
        env: Env,
        admin: Address,
        limit: u64,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if admin != storage::get_admin(&env)? {
            return Err(ContractError::Unauthorized);
        }
        admin.require_auth();
        storage::set_global_trade_limit(&env, limit);
        Ok(())
    }

    pub fn create_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        arbitrator: Option<Address>,
        metadata: OptionalMetadata,
        expiry_time: Option<u64>,
        currency: Option<Address>,
        metadata: Option<soroban_sdk::String>,
        trigger: Option<PriceTrigger>,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        validate_metadata(&metadata)?;
        seller.require_auth();
        validate_user_compliance(&env, &seller, amount)?;
        validate_user_compliance(&env, &buyer, amount)?;
        let arbitration = match arbitrator {
            Some(addr) => {
                if !storage::has_arbitrator(&env, &addr) {
                    return Err(ContractError::ArbitratorNotRegistered);
                }
                Some(addr)
            }
            None => None,
        };
        let trade_id = storage::increment_trade_counter(&env)?;
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
            trigger,
        };
        save_trade(&env, trade_id, &trade);
        events::emit_trade_created(&env, trade_id, seller.clone(), buyer.clone(), amount);
        events::emit_compliance_passed(&env, trade_id, seller, buyer, amount);
        analytics::on_trade_created(&env, amount, &trade.seller, &trade.buyer);
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
        metadata: Option<soroban_sdk::String>,
        trigger: Option<PriceTrigger>,
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

        if let Some(ref meta) = metadata {
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
            fee: calc_fee(&env, amount)?,
            arbitrator: arbitration,
            status: TradeStatus::Created,
            expiry_time: None,
            currency: storage::get_usdc_token(&env)?,
            metadata,
            trigger,
        };
        storage::save_trade(&env, trade_id, &trade);
        events::emit_trade_created(&env, trade_id, seller.clone(), buyer.clone(), amount, trade.currency.clone());
        events::emit_compliance_passed(&env, trade_id, seller, buyer, amount);
        analytics::on_trade_created(&env, amount, &trade.seller, &trade.buyer);
        Ok(trade_id)
    }

    pub fn fund_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Created {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();
        token::Client::new(&env, &trade.currency).transfer(
            &trade.buyer,
            &env.current_contract_address(),
            &(trade.amount as i128),
        );
        trade.status = TradeStatus::Funded;
        storage::save_trade(&env, trade_id, &trade);
        events::emit_trade_funded(&env, trade_id);
        analytics::on_trade_funded(&env);
        Ok(())
    }

    pub fn complete_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded {
            return Err(ContractError::InvalidStatus);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Completed;
        storage::save_trade(&env, trade_id, &trade);
        events::emit_trade_completed(&env, trade_id);
        Ok(())
    }

    pub fn confirm_receipt(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();
        let payout = trade
            .amount
            .checked_sub(trade.fee)
            .ok_or(ContractError::Overflow)?;
        let token_client = token::Client::new(&env, &trade.currency);
        let contract_balance = token_client.balance(&env.current_contract_address());
        let required_balance = trade.amount as i128;
        if contract_balance < required_balance {
            token_client.transfer(
                &trade.buyer,
                &env.current_contract_address(),
                &(required_balance - contract_balance),
            );
        }
        token_client.transfer(&env.current_contract_address(), &trade.seller, &(payout as i128));
        storage::add_accumulated_fees(&env, trade.fee)?;
        events::emit_trade_confirmed(&env, trade_id, payout, trade.fee);
        analytics::on_trade_completed(&env, trade.fee);
        Ok(())
    }

    pub fn cancel_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Created {
            return Err(ContractError::InvalidStatus);
    pub fn raise_dispute(env: Env, trade_id: u64, caller: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Cancelled;
        storage::save_trade(&env, trade_id, &trade);
        events::emit_trade_cancelled(&env, trade_id);
        analytics::on_trade_cancelled(&env);
        Ok(())
    }

    pub fn raise_dispute(
        env: Env,
        trade_id: u64,
        caller: Address,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded && trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        if trade.arbitrator.is_none() {
            return Err(ContractError::ArbitratorNotRegistered);
        }
        caller.require_auth();
        if caller != trade.buyer && caller != trade.seller {
            return Err(ContractError::Unauthorized);
        }
        trade.status = TradeStatus::Disputed;
        storage::save_trade(&env, trade_id, &trade);
        events::emit_dispute_raised(&env, trade_id, caller);
        analytics::on_trade_disputed(&env);
        Ok(())
    }

    /// Use `DisputeResolution::Partial { buyer_bps }` for a split:
    /// `buyer_bps` is the buyer's share of the net payout in basis points (0–10000).
    pub fn resolve_dispute(
        env: Env,
        trade_id: u64,
        resolution: DisputeResolution,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Disputed {
            return Err(ContractError::InvalidStatus);
        }
        let arbitrator = match trade.arbitrator.clone() {
            Some(addr) => addr,
            None => return Err(ContractError::NoArbitrator),
        };
        arbitrator.require_auth();
        let net = trade
            .amount
            .checked_sub(trade.fee)
            .ok_or(ContractError::Overflow)?;
        let token_client = token::Client::new(&env, &trade.currency);
        match resolution.clone() {
            DisputeResolution::ReleaseToBuyer => {
                token_client.transfer(&env.current_contract_address(), &trade.buyer, &(net as i128));
                events::emit_dispute_resolved(&env, trade_id, resolution, trade.buyer);
            }
            DisputeResolution::ReleaseToSeller => {
                token_client.transfer(&env.current_contract_address(), &trade.seller, &(net as i128));
                events::emit_dispute_resolved(&env, trade_id, resolution, trade.seller);
            }
            DisputeResolution::Partial(buyer_bps) => {
                if buyer_bps > 10_000 {
                    return Err(ContractError::InvalidSplitBps);
                }
                let buyer_amount = net
                    .checked_mul(buyer_bps as u64)
                    .ok_or(ContractError::Overflow)?
                    .checked_div(10_000)
                    .ok_or(ContractError::Overflow)?;
                let seller_amount = net
                    .checked_sub(buyer_amount)
                    .ok_or(ContractError::Overflow)?;
                if buyer_amount > 0 {
                    token_client.transfer(
                        &env.current_contract_address(),
                        &trade.buyer,
                        &(buyer_amount as i128),
                    );
                }
                if seller_amount > 0 {
                    token_client.transfer(
                        &env.current_contract_address(),
                        &trade.seller,
                        &(seller_amount as i128),
                    );
                }
                events::emit_partial_resolved(&env, trade_id, buyer_amount, seller_amount, trade.fee);
            }
        }
        storage::add_accumulated_fees(&env, trade.fee)?;
        Ok(())
    }

    pub fn get_trade(env: Env, trade_id: u64) -> Result<Trade, ContractError> {
        storage::get_trade(&env, trade_id)
    }

    pub fn withdraw_fees(env: Env, to: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = storage::get_admin(&env)?;
        admin.require_auth();
        let fees = storage::get_accumulated_fees(&env)?;
        if fees == 0 {
            return Err(ContractError::NoFeesToWithdraw);
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
        Ok(())
    }

    /// anyone can call this once the expiry has
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
        usdc_client(&env)?.transfer(&env.current_contract_address(), &to, &(fees as i128));
        storage::set_accumulated_fees(&env, 0);
        events::emit_fees_withdrawn(&env, fees, to);
        Ok(())
    }

    pub fn get_accumulated_fees(env: Env) -> Result<u64, ContractError> {
        storage::get_accumulated_fees(&env)
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

    /// Check and execute a price trigger for a trade.
    /// Can be called by anyone; trigger logic is automated based on oracle price.
    pub fn execute_price_trigger(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        let trigger = match &trade.trigger {
            Some(t) => t.clone(),
            None => return Err(ContractError::NoTrigger),
        };
        // Trigger can only execute for funded trades
        if trade.status != TradeStatus::Funded {
            return Err(ContractError::InvalidStatus);
        }

        if oracle::check_trigger(&env, &trigger)? {
            match trigger.action {
                TriggerAction::Cancel => {
                    // Refund entire escrowed amount to buyer
                    let token_client = token::Client::new(&env, &trade.currency);
                    token_client.transfer(
                        &env.current_contract_address(),
                        &trade.buyer,
                        &(trade.amount as i128),
                    );
                    trade.status = TradeStatus::Cancelled;
                }
                TriggerAction::Release => {
                    // Release to seller, minus platform fee
                    let token_client = token::Client::new(&env, &trade.currency);
                    let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
                    token_client.transfer(
                        &env.current_contract_address(),
                        &trade.seller,
                        &(payout as i128),
                    );
                    // Add fee to contract's accumulated revenue
                    let current_fees = storage::get_currency_fees(&env, &trade.currency);
                    let new_fees = current_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?;
                    storage::set_currency_fees(&env, &trade.currency, new_fees);
                    storage::add_accumulated_fees(&env, trade.fee)?;
                    trade.status = TradeStatus::Triggered;
                }
            }
            save_trade(&env, trade_id, &trade);
            events::emit_trigger_executed(&env, trade_id, &trigger.action);
        } else {
            return Err(ContractError::PriceConditionNotMet);
        }

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Emergency Pause
    // -------------------------------------------------------------------------

    /// Pause all contract operations (admin only).
    pub fn pause(env: Env) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = storage::get_admin(&env)?;
        admin.require_auth();
        storage::set_paused(&env, true);
        events::emit_paused(&env, admin);
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = storage::get_admin(&env)?;
        admin.require_auth();
        storage::set_paused(&env, false);
        events::emit_unpaused(&env, admin);
        Ok(())
    }

    /// Emergency withdrawal of all contract token balance (admin only).
    /// Allowed even while paused so funds can always be recovered.
    pub fn emergency_withdraw(env: Env, to: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);
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
        storage::is_paused(&env)
    }

    pub fn version(env: Env) -> u32 {
        storage::get_version(&env)
    }

    pub fn migrate(env: Env, expected_version: u32) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let current = storage::get_version(&env);
        if current != expected_version {
            return Err(ContractError::MigrationVersionMismatch);
        }
        let next = current.checked_add(1).ok_or(ContractError::Overflow)?;
        storage::set_version(&env, next);
        events::emit_migrated(&env, current, next);
        Ok(())
    }

    pub fn set_bridge_oracle(env: Env, oracle: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        storage::get_admin(&env)?.require_auth();
        storage::set_bridge_oracle(&env, &oracle);
        events::emit_bridge_oracle_set(&env, oracle);
        Ok(())
    }

    pub fn create_cross_chain_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        arbitrator: Option<Address>,
        source_chain: String,
        expiry_ledgers: u32,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        if storage::get_bridge_oracle(&env).is_none() {
            return Err(ContractError::BridgeOracleNotSet);
        }
        seller.require_auth();
        let arbitration = match arbitrator {
            Some(addr) => {
                if !storage::has_arbitrator(&env, &addr) {
                    return Err(ContractError::ArbitratorNotRegistered);
                }
                Some(addr)
            }
            None => None,
        };
        let trade_id = storage::increment_trade_counter(&env)?;
        let expires_at_ledger = env
            .ledger()
            .sequence()
            .checked_add(expiry_ledgers)
            .ok_or(ContractError::Overflow)?;
        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee: calc_fee(&env, amount)?,
            arbitrator: arbitration,
            status: TradeStatus::AwaitingBridge,
            expiry_time: None,
            currency: storage::get_usdc_token(&env)?,
            metadata: OptionalMetadata::None,
            currency: get_usdc_token(&env)?,
            metadata: None,
            trigger: None,
        };
        storage::save_trade(&env, trade_id, &trade);
        storage::save_cross_chain_info(
            &env,
            trade_id,
            &CrossChainInfo {
                source_chain: source_chain.clone(),
                source_tx_hash: String::from_str(&env, ""),
                expires_at_ledger,
            },
        );
        events::emit_bridge_trade_created(&env, trade_id, source_chain);
        analytics::on_trade_created(&env, amount, &trade.seller, &trade.buyer);
        Ok(trade_id)
    }

    pub fn confirm_bridge_deposit(
        env: Env,
        trade_id: u64,
        source_tx_hash: String,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let oracle = storage::get_bridge_oracle(&env).ok_or(ContractError::BridgeOracleNotSet)?;
        oracle.require_auth();
        let mut trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::AwaitingBridge {
            return Err(ContractError::InvalidStatus);
        }
        let mut info = storage::get_cross_chain_info(&env, trade_id).ok_or(ContractError::TradeNotFound)?;
        if env.ledger().sequence() > info.expires_at_ledger {
            return Err(ContractError::BridgeTradeExpired);
        }
        info.source_tx_hash = source_tx_hash;
        storage::save_cross_chain_info(&env, trade_id, &info);
        trade.status = TradeStatus::Funded;
        storage::save_trade(&env, trade_id, &trade);
        events::emit_bridge_deposit_confirmed(&env, trade_id);
        analytics::on_trade_funded(&env);
        Ok(())
    }

    pub fn expire_bridge_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let mut trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::AwaitingBridge {
            return Err(ContractError::InvalidStatus);
        }
        let info = storage::get_cross_chain_info(&env, trade_id).ok_or(ContractError::TradeNotFound)?;
        if env.ledger().sequence() <= info.expires_at_ledger {
            return Err(ContractError::BridgeTradeNotExpired);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Cancelled;
        storage::save_trade(&env, trade_id, &trade);
        events::emit_bridge_trade_expired(&env, trade_id);
        analytics::on_trade_cancelled(&env);
        Ok(())
    }

    pub fn get_cross_chain_info(env: Env, trade_id: u64) -> Option<CrossChainInfo> {
        storage::get_cross_chain_info(&env, trade_id)
    }

    pub fn register_insurance_provider(
        env: Env,
        provider: Address,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        storage::get_admin(&env)?.require_auth();
        storage::save_insurance_provider(&env, &provider);
        events::emit_insurance_provider_registered(&env, provider);
        Ok(())
    }

    pub fn remove_insurance_provider(env: Env, provider: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        storage::get_admin(&env)?.require_auth();
        storage::remove_insurance_provider(&env, &provider);
        events::emit_insurance_provider_removed(&env, provider);
        Ok(())
    }

    pub fn is_insurance_provider_registered(env: Env, provider: Address) -> bool {
        storage::has_insurance_provider(&env, &provider)
    }

    pub fn purchase_insurance(
        env: Env,
        trade_id: u64,
        provider: Address,
        premium_bps: u32,
        coverage: u64,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if premium_bps > MAX_INSURANCE_PREMIUM_BPS {
            return Err(ContractError::InsurancePremiumTooHigh);
        }
        if !storage::has_insurance_provider(&env, &provider) {
            return Err(ContractError::InsuranceProviderNotRegistered);
        }
        let trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded && trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();
        let premium = trade
            .amount
            .checked_mul(premium_bps as u64)
            .ok_or(ContractError::Overflow)?
            .checked_div(10_000)
            .ok_or(ContractError::Overflow)?;
        usdc_client(&env)?.transfer(&trade.buyer, &provider, &(premium as i128));
        storage::save_insurance_policy(
            &env,
            trade_id,
            &InsurancePolicy {
                provider: provider.clone(),
                premium,
                coverage,
                claimed: false,
            },
        );
        events::emit_insurance_purchased(&env, trade_id, provider, premium, coverage);
        Ok(())
    }

    pub fn claim_insurance(
        env: Env,
        trade_id: u64,
        recipient: Address,
        payout: u64,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Disputed && trade.status != TradeStatus::Completed {
            return Err(ContractError::InsuranceClaimNotEligible);
        }
        let mut policy = storage::get_insurance_policy(&env, trade_id).ok_or(ContractError::TradeNotInsured)?;
        if policy.claimed {
            return Err(ContractError::InsuranceAlreadyClaimed);
        }
        policy.provider.require_auth();
        let actual_payout = if payout > policy.coverage { policy.coverage } else { payout };
        usdc_client(&env)?.transfer(&policy.provider, &recipient, &(actual_payout as i128));
        policy.claimed = true;
        storage::save_insurance_policy(&env, trade_id, &policy);
        events::emit_insurance_claimed(&env, trade_id, actual_payout, recipient);
        Ok(())
    }

    pub fn get_insurance_policy(env: Env, trade_id: u64) -> Option<InsurancePolicy> {
        storage::get_insurance_policy(&env, trade_id)
    }

    pub fn get_platform_metrics(env: Env) -> PlatformMetrics {
        analytics::get_metrics(&env)
    }

    pub fn get_platform_stats(env: Env) -> PlatformStats {
        analytics::get_stats(&env)
    }

    pub fn get_arbitrator_analytics(env: Env, arbitrator: Address) -> ArbitratorMetrics {
        analytics::get_arb_metrics(&env, &arbitrator)
    }

    pub fn analytics_query(env: Env, window: TimeWindow) -> AnalyticsResult {
        analytics::analytics_query(&env, window)
    }

    pub fn get_volume_stats(env: Env, window: TimeWindow) -> VolumeStats {
        analytics::get_volume_stats(&env, window)
    }

    pub fn get_success_rate(env: Env) -> SuccessRateStats {
        analytics::get_success_rate(&env)
    }

    pub fn get_platform_usage(env: Env) -> analytics::PlatformUsage {
        analytics::get_platform_usage(&env)
    }

    pub fn get_analytics_by_period(env: Env, start_time: u64, end_time: u64) -> PeriodAnalytics {
        analytics::get_analytics_by_period(&env, start_time, end_time)
    }
}

#[cfg(test)]
mod test;
