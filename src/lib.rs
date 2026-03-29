#![no_std]

#[cfg(test)]
extern crate std;

mod admin;
mod analytics;
mod audit;
mod errors;
mod events;
mod filtering;
mod history;
mod onboarding;
mod storage;
mod templates;
mod theme;
mod tiers;
mod trade_detail;
mod trade_form;
mod fund_trade;
mod types;
mod users;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};
use soroban_sdk::token::TokenClient;

use types::{METADATA_MAX_ENTRIES, METADATA_MAX_VALUE_LEN};

pub use errors::ContractError;
pub use theme::{
    FONT_LG, FONT_MD, FONT_SM,
    PREF_FONT_SIZE, PREF_THEME_COLOR, PREF_THEME_MODE,
    THEME_DARK, THEME_LIGHT, THEME_SYSTEM,
};
pub use types::{
    AnalyticsFilter, ChartPoint, Currency, DashboardStats, DisputeResolution, FeeChartData,
    FilterPreset, FundingPreview, HistoryFilter, HistoryPage, MetadataEntry,
    OnboardingProgress, OnboardingStep, PlatformAnalytics, SortCriterion, SortOrder,
    StatusDistribution, StepStatus, SuccessRateData, SystemConfig, TierConfig, Trade,
    TradeAction, TradeDetail, TradeFilter, TradeFormInput, TradeMetadata, TradePreview,
    TradeSearchPage, TradeSortField, TradeStatus, TradeTemplate, TemplateTerms,
    TemplateVersion, TimelineEntry, TransactionRecord, UserAnalytics, UserPreference,
    UserProfile, UserStatsSnapshot, UserTier, UserTierInfo, VerificationStatus,
    VolumeChartData, VolumeInRange,
};

use storage::{
    add_currency_fees, append_timeline_entry, get_accumulated_fees, get_admin, get_fee_bps,
    get_currency_fees, get_trade, get_usdc_token, has_arbitrator, increment_trade_counter,
    index_trade_for_address, is_initialized, is_paused, remove_arbitrator, save_arbitrator,
    save_trade, set_accumulated_fees, set_admin, set_fee_bps, set_initialized, set_paused,
    set_trade_counter, set_usdc_token,
};

/// Return ContractPaused if the contract is currently paused.
fn require_not_paused(env: &Env) -> Result<(), ContractError> {
    if is_paused(env) {
        return Err(ContractError::ContractPaused);
    }
    Ok(())
}

fn validate_metadata(meta: &TradeMetadata) -> Result<(), ContractError> {
    if meta.entries.len() > METADATA_MAX_ENTRIES {
        return Err(ContractError::MetadataTooManyEntries);
    }
    for entry in meta.entries.iter() {
        if entry.value.len() > METADATA_MAX_VALUE_LEN {
            return Err(ContractError::MetadataValueTooLong);
        }
    }
    Ok(())
}

/// Internal helper used by [`trade_form::confirm_trade`] to create a trade
/// without going through the public contract entry-point (which requires
/// `seller.require_auth()` — already enforced by the caller).
pub(crate) fn lib_create_trade(
    env: &Env,
    seller: Address,
    buyer: Address,
    amount: u64,
    currency: types::Currency,
    expiry_time: Option<u64>,
    arbitrator: Option<Address>,
    metadata: Option<TradeMetadata>,
) -> Result<u64, ContractError> {
    use types::TimelineEntry;

    if !is_initialized(env) {
        return Err(ContractError::NotInitialized);
    }
    if is_paused(env) {
        return Err(ContractError::ContractPaused);
    }
    if amount == 0 {
        return Err(ContractError::InvalidAmount);
    }
    if let Some(ref arb) = arbitrator {
        if !has_arbitrator(env, arb) {
            return Err(ContractError::ArbitratorNotRegistered);
        }
    }
    if let Some(ref meta) = metadata {
        validate_metadata(meta)?;
    }
    let trade_id = increment_trade_counter(env)?;
    let base_fee_bps = get_fee_bps(env)?;
    let fee_bps = tiers::effective_fee_bps(env, &seller, base_fee_bps);
    let fee = amount
        .checked_mul(fee_bps as u64)
        .ok_or(ContractError::Overflow)?
        .checked_div(10_000)
        .ok_or(ContractError::Overflow)?;
    let now = env.ledger().sequence();
    let trade = Trade {
        id: trade_id,
        seller: seller.clone(),
        buyer: buyer.clone(),
        amount,
        fee,
        arbitrator,
        status: TradeStatus::Created,
        created_at: now,
        updated_at: now,
        metadata,
        currency: currency.clone(),
        expiry_time,
    };
    save_trade(env, trade_id, &trade);
    index_trade_for_address(env, &seller, trade_id);
    index_trade_for_address(env, &buyer, trade_id);
    append_timeline_entry(env, trade_id, TimelineEntry { status: TradeStatus::Created, ledger: now });
    // Accumulate fees globally and per-currency
    let current_fees = get_accumulated_fees(env)?;
    set_accumulated_fees(env, current_fees.checked_add(fee).ok_or(ContractError::Overflow)?);
    add_currency_fees(env, &currency, fee)?;
    users::record_trade_created(env, &seller, &buyer, amount);
    admin::on_trade_created(env, amount);
    events::emit_trade_created(env, trade_id, seller, buyer, amount);
    Ok(trade_id)
}

#[contract]
pub struct StellarEscrowContract;

#[contractimpl]
impl StellarEscrowContract {
    // -------------------------------------------------------------------------
    // Initialization
    // -------------------------------------------------------------------------

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
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Arbitrators
    // -------------------------------------------------------------------------

    pub fn register_arbitrator(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        save_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_registered(&env, arbitrator);
        Ok(())
    }

    pub fn remove_arbitrator_fn(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        remove_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_removed(&env, arbitrator);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Fee management
    // -------------------------------------------------------------------------

    pub fn update_fee(env: Env, fee_bps: u32) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        if fee_bps > 10000 { return Err(ContractError::InvalidFeeBps); }
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_fee_bps(&env, fee_bps);
        events::emit_fee_updated(&env, fee_bps);
        Ok(())
    }

    pub fn withdraw_fees(env: Env, to: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        let admin = get_admin(&env)?;
        admin.require_auth();
        let fees = get_accumulated_fees(&env)?;
        if fees == 0 { return Err(ContractError::NoFeesToWithdraw); }
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &to, &(fees as i128));
        set_accumulated_fees(&env, 0);
        events::emit_fees_withdrawn(&env, fees, to.clone());
        let _ = audit::log_admin(&env, admin, soroban_sdk::String::from_str(&env, "admin.fees_withdrawn"));
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Trades
    // -------------------------------------------------------------------------

    pub fn create_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        currency: types::Currency,
        expiry_time: Option<u64>,
        arbitrator: Option<Address>,
        metadata: Option<TradeMetadata>,
    ) -> Result<u64, ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        if amount == 0 { return Err(ContractError::InvalidAmount); }
        seller.require_auth();
        if let Some(ref arb) = arbitrator {
            if !has_arbitrator(&env, arb) { return Err(ContractError::ArbitratorNotRegistered); }
        }
        if let Some(ref meta) = metadata { validate_metadata(meta)?; }
        let trade_id = increment_trade_counter(&env)?;
        let base_fee_bps = get_fee_bps(&env)?;
        let fee_bps = tiers::effective_fee_bps(&env, &seller, base_fee_bps);
        let fee = amount.checked_mul(fee_bps as u64).ok_or(ContractError::Overflow)?
            .checked_div(10000).ok_or(ContractError::Overflow)?;
        let now = env.ledger().sequence();
        let trade = Trade {
            id: trade_id, seller: seller.clone(), buyer: buyer.clone(),
            amount, fee, arbitrator, status: TradeStatus::Created,
            created_at: now, updated_at: now, metadata,
            currency: currency.clone(),
            expiry_time,
        };
        save_trade(&env, trade_id, &trade);
        index_trade_for_address(&env, &seller, trade_id);
        index_trade_for_address(&env, &buyer, trade_id);
        append_timeline_entry(&env, trade_id, TimelineEntry { status: TradeStatus::Created, ledger: now });
        // Accumulate fees globally and per-currency
        let current_fees = get_accumulated_fees(&env)?;
        set_accumulated_fees(&env, current_fees.checked_add(fee).ok_or(ContractError::Overflow)?);
        add_currency_fees(&env, &currency, fee)?;
        users::record_trade_created(&env, &seller, &buyer, amount);
        admin::on_trade_created(&env, amount);
        events::emit_trade_created(&env, trade_id, seller.clone(), buyer, amount);
        let _ = audit::log_trade(&env, seller, soroban_sdk::String::from_str(&env, "trade.created"), trade_id);
        Ok(trade_id)
    }

    pub fn fund_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Created { return Err(ContractError::InvalidStatus); }
        // Reject funding an already-expired trade
        if let Some(expiry) = trade.expiry_time {
            if env.ledger().timestamp() >= expiry {
                return Err(ContractError::TradeExpired);
            }
        }
        trade.buyer.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&trade.buyer, &env.current_contract_address(), &(trade.amount as i128));
        trade.status = TradeStatus::Funded;
        trade.updated_at = env.ledger().sequence();
        save_trade(&env, trade_id, &trade);
        append_timeline_entry(&env, trade_id, TimelineEntry { status: TradeStatus::Funded, ledger: trade.updated_at });
        events::emit_trade_funded(&env, trade_id);
        Ok(())
    }

    pub fn complete_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded { return Err(ContractError::InvalidStatus); }
        // Block manual completion if the trade has expired — use auto_release instead
        if let Some(expiry) = trade.expiry_time {
            if env.ledger().timestamp() >= expiry {
                return Err(ContractError::TradeExpired);
            }
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Completed;
        trade.updated_at = env.ledger().sequence();
        save_trade(&env, trade_id, &trade);
        append_timeline_entry(&env, trade_id, TimelineEntry { status: TradeStatus::Completed, ledger: trade.updated_at });
        events::emit_trade_completed(&env, trade_id);
        Ok(())
    }

    /// Automatically release funds to the seller when `expiry_time` has passed
    /// and the trade is still Funded with no active dispute.
    /// Anyone may call this — no auth required (permissionless trigger).
    pub fn auto_release(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded { return Err(ContractError::InvalidStatus); }
        let expiry = trade.expiry_time.ok_or(ContractError::NotExpiredYet)?;
        let now_ts = env.ledger().timestamp(); // UTC Unix seconds — no timezone conversion needed
        if now_ts < expiry {
            return Err(ContractError::NotExpiredYet);
        }
        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &trade.seller, &(payout as i128));
        let current_fees = get_accumulated_fees(&env)?;
        set_accumulated_fees(&env, current_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?);
        tiers::record_volume(&env, &trade.seller, trade.amount)?;
        tiers::record_volume(&env, &trade.buyer, trade.amount)?;
        users::record_trade_completed(&env, &trade.seller, &trade.buyer);
        admin::on_trade_completed(&env, trade.fee);
        trade.status = TradeStatus::Completed;
        trade.updated_at = env.ledger().sequence();
        save_trade(&env, trade_id, &trade);
        append_timeline_entry(&env, trade_id, TimelineEntry { status: TradeStatus::Completed, ledger: trade.updated_at });
        events::emit_trade_auto_released(&env, trade_id, payout, now_ts);
        Ok(())
    }

    pub fn confirm_receipt(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        let trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Completed { return Err(ContractError::InvalidStatus); }
        trade.buyer.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
        token_client.transfer(&env.current_contract_address(), &trade.seller, &(payout as i128));
        let current_fees = get_accumulated_fees(&env)?;
        let new_fees = current_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?;
        set_accumulated_fees(&env, new_fees);
        tiers::record_volume(&env, &trade.seller, trade.amount)?;
        tiers::record_volume(&env, &trade.buyer, trade.amount)?;
        users::record_trade_completed(&env, &trade.seller, &trade.buyer);
        admin::on_trade_completed(&env, trade.fee);
        events::emit_trade_confirmed(&env, trade_id, payout, trade.fee);
        Ok(())
    }

    pub fn raise_dispute(env: Env, trade_id: u64, caller: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded && trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        // Cannot raise a dispute after expiry — auto_release should be called instead
        if let Some(expiry) = trade.expiry_time {
            if env.ledger().timestamp() >= expiry {
                return Err(ContractError::TradeExpired);
            }
        }
        if trade.arbitrator.is_none() { return Err(ContractError::ArbitratorNotRegistered); }
        if caller != trade.buyer && caller != trade.seller { return Err(ContractError::Unauthorized); }
        caller.require_auth();
        trade.status = TradeStatus::Disputed;
        trade.updated_at = env.ledger().sequence();
        save_trade(&env, trade_id, &trade);
        append_timeline_entry(&env, trade_id, TimelineEntry { status: TradeStatus::Disputed, ledger: trade.updated_at });
        users::record_trade_disputed(&env, &trade.seller, &trade.buyer);
        admin::on_trade_disputed(&env);
        events::emit_dispute_raised(&env, trade_id, caller);
        Ok(())
    }

    /// Resolve a dispute (arbitrator only)
    pub fn resolve_dispute(env: Env, trade_id: u64, resolution: DisputeResolution) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        let trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Disputed { return Err(ContractError::InvalidStatus); }
        let arbitrator = trade.arbitrator.ok_or(ContractError::ArbitratorNotRegistered)?;
        arbitrator.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        let recipient = match resolution {
            DisputeResolution::ReleaseToBuyer => trade.buyer.clone(),
            DisputeResolution::ReleaseToSeller => trade.seller.clone(),
        };
        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
        token_client.transfer(&env.current_contract_address(), &recipient, &(payout as i128));
        let current_fees = get_accumulated_fees(&env)?;
        set_accumulated_fees(&env, current_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?);
        events::emit_dispute_resolved(&env, trade_id, resolution, recipient.clone());
        let _ = audit::log_trade(&env, arbitrator, soroban_sdk::String::from_str(&env, "dispute.resolved"), trade_id);
        Ok(())
    }

    pub fn cancel_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Created { return Err(ContractError::InvalidStatus); }
        trade.seller.require_auth();
        trade.status = TradeStatus::Cancelled;
        trade.updated_at = env.ledger().sequence();
        save_trade(&env, trade_id, &trade);
        append_timeline_entry(&env, trade_id, TimelineEntry { status: TradeStatus::Cancelled, ledger: trade.updated_at });
        users::record_trade_cancelled(&env, &trade.seller);
        admin::on_trade_cancelled(&env);
        events::emit_trade_cancelled(&env, trade_id);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Query functions
    // -------------------------------------------------------------------------

    pub fn get_trade(env: Env, trade_id: u64) -> Result<Trade, ContractError> {
        get_trade(&env, trade_id)
    }

    pub fn get_accumulated_fees(env: Env) -> Result<u64, ContractError> {
        get_accumulated_fees(&env)
    }

    /// Query accumulated fees for a specific currency.
    /// Returns 0 if no fees have been collected for that currency yet.
    pub fn get_currency_fees(env: Env, currency: types::Currency) -> u64 {
        get_currency_fees(&env, &currency)
    }

    pub fn is_arbitrator_registered(env: Env, arbitrator: Address) -> bool {
        has_arbitrator(&env, &arbitrator)
    }

    pub fn get_platform_fee_bps(env: Env) -> Result<u32, ContractError> {
        get_fee_bps(&env)
    }

    pub fn update_trade_metadata(env: Env, trade_id: u64, metadata: Option<TradeMetadata>) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        let mut trade = get_trade(&env, trade_id)?;
        trade.seller.require_auth();
        if let Some(ref meta) = metadata { validate_metadata(meta)?; }
        trade.metadata = metadata;
        trade.updated_at = env.ledger().sequence();
        save_trade(&env, trade_id, &trade);
        events::emit_metadata_updated(&env, trade_id);
        Ok(())
    }

    pub fn get_trade_metadata(env: Env, trade_id: u64) -> Result<Option<TradeMetadata>, ContractError> {
        let trade = get_trade(&env, trade_id)?;
        Ok(trade.metadata)
    }

    // -------------------------------------------------------------------------
    // Pause / emergency
    // -------------------------------------------------------------------------

    pub fn pause(env: Env) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_paused(&env, true);
        events::emit_paused(&env, admin.clone());
        let _ = audit::log_admin(&env, admin, soroban_sdk::String::from_str(&env, "contract.pause"));
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_paused(&env, false);
        events::emit_unpaused(&env, admin.clone());
        let _ = audit::log_admin(&env, admin, soroban_sdk::String::from_str(&env, "contract.unpause"));
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        is_paused(&env)
    }

    pub fn emergency_withdraw(env: Env, to: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
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
        let _ = audit::log_admin(&env, admin, soroban_sdk::String::from_str(&env, "admin.emergency_withdraw"));
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Batch operations
    // -------------------------------------------------------------------------

    pub fn batch_create_trades(
        env: Env,
        seller: Address,
        trades: soroban_sdk::Vec<(Address, u64, Option<Address>)>,
    ) -> Result<soroban_sdk::Vec<u64>, ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        if trades.is_empty() { return Err(ContractError::EmptyBatch); }
        if trades.len() > 100 { return Err(ContractError::BatchLimitExceeded); }
        seller.require_auth();
        let base_fee_bps = get_fee_bps(&env)?;
        let fee_bps = tiers::effective_fee_bps(&env, &seller, base_fee_bps);
        let mut trade_ids = soroban_sdk::Vec::new(&env);
        let mut total_amount: u64 = 0;
        let mut total_fees: u64 = 0;
        let now = env.ledger().sequence();
        let currency = types::Currency::Usdc;
        for (buyer, amount, arbitrator) in trades.iter() {
            if amount == 0 { return Err(ContractError::InvalidAmount); }
            if let Some(ref arb) = arbitrator {
                if !has_arbitrator(&env, arb) { return Err(ContractError::ArbitratorNotRegistered); }
            }
            let trade_id = increment_trade_counter(&env)?;
            let fee = amount.checked_mul(fee_bps as u64).ok_or(ContractError::Overflow)?
                .checked_div(10000).ok_or(ContractError::Overflow)?;
            let trade = Trade {
                id: trade_id, seller: seller.clone(), buyer: buyer.clone(),
                amount, fee, arbitrator, status: TradeStatus::Created,
                created_at: now, updated_at: now, metadata: None,
                currency: currency.clone(), expiry_time: None,
            };
            save_trade(&env, trade_id, &trade);
            index_trade_for_address(&env, &seller, trade_id);
            index_trade_for_address(&env, &buyer, trade_id);
            append_timeline_entry(&env, trade_id, TimelineEntry { status: TradeStatus::Created, ledger: now });
            users::record_trade_created(&env, &seller, &buyer, amount);
            admin::on_trade_created(&env, amount);
            trade_ids.push_back(trade_id);
            total_amount = total_amount.checked_add(amount).ok_or(ContractError::Overflow)?;
            total_fees = total_fees.checked_add(fee).ok_or(ContractError::Overflow)?;
        }
        let current_fees = get_accumulated_fees(&env)?;
        set_accumulated_fees(&env, current_fees.checked_add(total_fees).ok_or(ContractError::Overflow)?);
        add_currency_fees(&env, &currency, total_fees)?;
        events::emit_batch_trades_created(&env, trade_ids.len() as u32, total_amount);
        Ok(trade_ids)
    }

    pub fn batch_fund_trades(
        env: Env,
        buyer: Address,
        trade_ids: soroban_sdk::Vec<u64>,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        if trade_ids.is_empty() { return Err(ContractError::EmptyBatch); }
        if trade_ids.len() > 100 { return Err(ContractError::BatchLimitExceeded); }
        buyer.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        let mut total_amount: u64 = 0;
        let mut seen: soroban_sdk::Map<u64, bool> = soroban_sdk::Map::new(&env);
        let now_ts = env.ledger().timestamp();
        for trade_id in trade_ids.iter() {
            if seen.get(trade_id).is_some() {
                return Err(ContractError::DuplicateTradeInBatch);
            }
            seen.set(trade_id, true);
            let trade = get_trade(&env, trade_id)?;
            if trade.status != TradeStatus::Created { return Err(ContractError::InvalidStatus); }
            if trade.buyer != buyer { return Err(ContractError::Unauthorized); }
            if let Some(expiry) = trade.expiry_time {
                if now_ts >= expiry {
                    return Err(ContractError::TradeExpired);
                }
            }
            total_amount = total_amount.checked_add(trade.amount).ok_or(ContractError::Overflow)?;
        }
        token_client.transfer(&buyer, &env.current_contract_address(), &(total_amount as i128));
        let now = env.ledger().sequence();
        for trade_id in trade_ids.iter() {
            let mut trade = get_trade(&env, trade_id)?;
            trade.status = TradeStatus::Funded;
            trade.updated_at = now;
            save_trade(&env, trade_id, &trade);
            append_timeline_entry(&env, trade_id, TimelineEntry { status: TradeStatus::Funded, ledger: now });
        }
        events::emit_batch_trades_funded(&env, trade_ids.len() as u32, total_amount);
        Ok(())
    }

    pub fn batch_confirm_trades(
        env: Env,
        buyer: Address,
        trade_ids: soroban_sdk::Vec<u64>,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        require_not_paused(&env)?;
        if trade_ids.is_empty() { return Err(ContractError::EmptyBatch); }
        if trade_ids.len() > 100 { return Err(ContractError::BatchLimitExceeded); }
        buyer.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        let mut total_payout: u64 = 0;
        let mut total_fees: u64 = 0;
        let mut seller_payouts: soroban_sdk::Map<Address, u64> = soroban_sdk::Map::new(&env);
        let mut seen: soroban_sdk::Map<u64, bool> = soroban_sdk::Map::new(&env);
        for trade_id in trade_ids.iter() {
            if seen.get(trade_id).is_some() {
                return Err(ContractError::DuplicateTradeInBatch);
            }
            seen.set(trade_id, true);
            let trade = get_trade(&env, trade_id)?;
            if trade.status != TradeStatus::Completed { return Err(ContractError::InvalidStatus); }
            if trade.buyer != buyer { return Err(ContractError::Unauthorized); }
            let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
            total_payout = total_payout.checked_add(payout).ok_or(ContractError::Overflow)?;
            total_fees = total_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?;
            let current = seller_payouts.get(trade.seller.clone()).unwrap_or(0);
            seller_payouts.set(trade.seller.clone(), current.checked_add(payout).ok_or(ContractError::Overflow)?);
        }
        for (seller, payout) in seller_payouts.iter() {
            token_client.transfer(&env.current_contract_address(), &seller, &(payout as i128));
        }
        let current_fees = get_accumulated_fees(&env)?;
        set_accumulated_fees(&env, current_fees.checked_add(total_fees).ok_or(ContractError::Overflow)?);
        for trade_id in trade_ids.iter() {
            let trade = get_trade(&env, trade_id)?;
            tiers::record_volume(&env, &trade.seller, trade.amount)?;
            tiers::record_volume(&env, &trade.buyer, trade.amount)?;
            users::record_trade_completed(&env, &trade.seller, &trade.buyer);
            admin::on_trade_completed(&env, trade.fee);
        }
        events::emit_batch_trades_confirmed(&env, trade_ids.len() as u32, total_payout, total_fees);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Fee Tier System
    // -------------------------------------------------------------------------

    pub fn set_tier_config(env: Env, config: TierConfig) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        let admin = get_admin(&env)?;
        admin.require_auth();
        tiers::set_tier_config(&env, &config)
    }

    pub fn set_user_custom_fee(env: Env, user: Address, fee_bps: u32) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        let admin = get_admin(&env)?;
        admin.require_auth();
        tiers::set_custom_fee(&env, &user, fee_bps)
    }

    pub fn remove_user_custom_fee(env: Env, user: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        let admin = get_admin(&env)?;
        admin.require_auth();
        tiers::remove_custom_fee(&env, &user);
        Ok(())
    }

    pub fn get_user_tier(env: Env, user: Address) -> Option<UserTierInfo> {
        storage::get_user_tier(&env, &user)
    }

    pub fn get_tier_config(env: Env) -> Option<TierConfig> {
        storage::get_tier_config(&env)
    }

    pub fn get_effective_fee_bps(env: Env, user: Address) -> Result<u32, ContractError> {
        let base = get_fee_bps(&env)?;
        Ok(tiers::effective_fee_bps(&env, &user, base))
    }

    // -------------------------------------------------------------------------
    // Trade Templates
    // -------------------------------------------------------------------------

    pub fn create_template(env: Env, owner: Address, name: soroban_sdk::String, terms: TemplateTerms) -> Result<u64, ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        owner.require_auth();
        templates::create_template(&env, &owner, name, terms)
    }

    pub fn update_template(env: Env, caller: Address, template_id: u64, name: soroban_sdk::String, terms: TemplateTerms) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        caller.require_auth();
        templates::update_template(&env, &caller, template_id, name, terms)
    }

    pub fn deactivate_template(env: Env, caller: Address, template_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        caller.require_auth();
        templates::deactivate_template(&env, &caller, template_id)
    }

    pub fn create_trade_from_template(env: Env, seller: Address, buyer: Address, template_id: u64, amount: u64) -> Result<u64, ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        if amount == 0 { return Err(ContractError::InvalidAmount); }
        seller.require_auth();
        let (terms, version) = templates::resolve_terms(&env, template_id)?;
        if let Some(fixed) = terms.fixed_amount {
            if amount != fixed { return Err(ContractError::TemplateAmountMismatch); }
        }
        if let Some(ref arb) = terms.default_arbitrator {
            if !has_arbitrator(&env, arb) { return Err(ContractError::ArbitratorNotRegistered); }
        }
        let trade_id = increment_trade_counter(&env)?;
        let base_fee_bps = get_fee_bps(&env)?;
        let fee_bps = tiers::effective_fee_bps(&env, &seller, base_fee_bps);
        let fee = amount.checked_mul(fee_bps as u64).ok_or(ContractError::Overflow)?
            .checked_div(10000).ok_or(ContractError::Overflow)?;
        let now = env.ledger().sequence();
        let trade = Trade {
            id: trade_id, seller: seller.clone(), buyer: buyer.clone(),
            amount, fee, arbitrator: terms.default_arbitrator,
            status: TradeStatus::Created, created_at: now, updated_at: now,
            metadata: terms.default_metadata,
            currency: types::Currency::Usdc, expiry_time: None,
        };
        save_trade(&env, trade_id, &trade);
        index_trade_for_address(&env, &seller, trade_id);
        index_trade_for_address(&env, &buyer, trade_id);
        events::emit_trade_created(&env, trade_id, seller, buyer, amount);
        events::emit_trade_from_template(&env, trade_id, template_id, version);
        Ok(trade_id)
    }

    pub fn get_template(env: Env, template_id: u64) -> Result<TradeTemplate, ContractError> {
        storage::get_template(&env, template_id)
    }

    // -------------------------------------------------------------------------
    // Transaction history
    // -------------------------------------------------------------------------

    pub fn get_transaction_history(env: Env, address: Address, filter: HistoryFilter, sort: SortOrder, offset: u32, limit: u32) -> Result<HistoryPage, ContractError> {
        history::get_history(&env, address, filter, sort, offset, limit)
    }

    pub fn export_transaction_csv(env: Env, address: Address, filter: HistoryFilter) -> Result<soroban_sdk::String, ContractError> {
        history::export_csv(&env, address, filter)
    }

    // -------------------------------------------------------------------------
    // User Management
    // -------------------------------------------------------------------------

    pub fn register_user(env: Env, address: Address, username_hash: soroban_sdk::Bytes, contact_hash: soroban_sdk::Bytes) -> Result<(), ContractError> {
        users::register_user(&env, address, username_hash, contact_hash)
    }

    pub fn update_profile(env: Env, address: Address, username_hash: soroban_sdk::Bytes, contact_hash: soroban_sdk::Bytes) -> Result<(), ContractError> {
        users::update_profile(&env, address, username_hash, contact_hash)
    }

    pub fn get_user_profile(env: Env, address: Address) -> Result<UserProfile, ContractError> {
        users::get_profile(&env, &address)
    }

    pub fn set_user_preference(env: Env, address: Address, key: soroban_sdk::String, value: soroban_sdk::String) -> Result<(), ContractError> {
        users::set_preference(&env, address, key, value)
    }

    pub fn get_user_preference(env: Env, address: Address, key: soroban_sdk::String) -> Result<UserPreference, ContractError> {
        users::get_pref(&env, &address, &key)
    }

    pub fn set_user_verification(env: Env, address: Address, status: VerificationStatus) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        let admin = get_admin(&env)?;
        admin.require_auth();
        users::set_verification(&env, &address, status)
    }

    pub fn get_user_analytics(env: Env, address: Address) -> UserAnalytics {
        users::get_user_analytics(&env, &address)
    }

    /// Update the avatar hash for a user (SHA-256 of the off-chain image).
    /// Pass `None` to remove the avatar.
    pub fn update_avatar(
        env: Env,
        address: Address,
        avatar_hash: Option<soroban_sdk::Bytes>,
    ) -> Result<(), ContractError> {
        users::update_avatar(&env, address, avatar_hash)
    }

    /// Update security settings: 2FA flag and session timeout (seconds).
    pub fn update_security_settings(
        env: Env,
        address: Address,
        two_fa_enabled: bool,
        session_timeout_secs: u32,
    ) -> Result<(), ContractError> {
        users::update_security_settings(&env, address, two_fa_enabled, session_timeout_secs)
    }

    // -------------------------------------------------------------------------
    // Admin Panel
    // -------------------------------------------------------------------------

    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        let current_admin = get_admin(&env)?;
        admin::transfer_admin(&env, current_admin, new_admin)
    }

    pub fn pause_contract(env: Env) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        let a = get_admin(&env)?;
        a.require_auth();
        admin::pause_contract(&env);
        Ok(())
    }

    pub fn unpause_contract(env: Env) -> Result<(), ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        let a = get_admin(&env)?;
        a.require_auth();
        admin::unpause_contract(&env);
        Ok(())
    }

    pub fn get_platform_analytics(env: Env) -> Result<PlatformAnalytics, ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        Ok(admin::get_analytics(&env))
    }

    /// Get full dashboard snapshot: platform stats, success rate, dispute rate, avg volume.
    pub fn get_dashboard(env: Env) -> Result<DashboardStats, ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        Ok(admin::get_dashboard(&env))
    }

    /// Get trade volume and counts for an address within a ledger range (date-range charts).
    pub fn get_volume_in_range(
        env: Env,
        address: Address,
        from_ledger: u32,
        to_ledger: u32,
    ) -> Result<VolumeInRange, ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        admin::get_volume_in_range(&env, &address, from_ledger, to_ledger)
    }

    /// Get system configuration snapshot (fee, pause state, counters).
    pub fn get_system_config(env: Env) -> Result<SystemConfig, ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        let a = get_admin(&env)?;
        a.require_auth();
        admin::get_system_config(&env)
    }

    // -------------------------------------------------------------------------
    // Trade Detail View
    // -------------------------------------------------------------------------

    pub fn get_trade_detail(env: Env, trade_id: u64, viewer: Address) -> Result<TradeDetail, ContractError> {
        trade_detail::get_trade_detail(&env, trade_id, viewer)
    }

    pub fn export_trade_csv(env: Env, trade_id: u64) -> Result<soroban_sdk::String, ContractError> {
        trade_detail::export_trade_csv(&env, trade_id)
    }

    // -------------------------------------------------------------------------
    // Analytics Charts & Graphs
    // -------------------------------------------------------------------------

    /// Get trade volume chart data bucketed by ledger range.
    pub fn get_volume_chart(env: Env, filter: AnalyticsFilter) -> Result<VolumeChartData, ContractError> {
        analytics::get_volume_chart(&env, filter)
    }

    /// Get platform-wide trade success rate.
    pub fn get_success_rate(env: Env) -> SuccessRateData {
        analytics::get_success_rate(&env)
    }

    /// Get trade status distribution breakdown.
    pub fn get_status_distribution(env: Env, filter: AnalyticsFilter) -> Result<StatusDistribution, ContractError> {
        analytics::get_status_distribution(&env, filter)
    }

    /// Get fee collection chart data bucketed by ledger range.
    pub fn get_fee_chart(env: Env, filter: AnalyticsFilter) -> Result<FeeChartData, ContractError> {
        analytics::get_fee_chart(&env, filter)
    }

    /// Get aggregated stats snapshot for a single user.
    pub fn get_user_stats(env: Env, address: Address) -> UserStatsSnapshot {
        analytics::get_user_stats(&env, &address)
    }

    /// Get per-user volume chart from their trade history.
    pub fn get_user_volume_chart(env: Env, address: Address, filter: AnalyticsFilter) -> Result<VolumeChartData, ContractError> {
        analytics::get_user_volume_chart(&env, &address, filter)
    }

    /// Export platform analytics as CSV (no PII).
    pub fn export_platform_analytics_csv(env: Env) -> soroban_sdk::String {
        analytics::export_platform_csv(&env)
    }

    /// Export volume chart data as CSV (no PII).
    pub fn export_volume_chart_csv(env: Env, filter: AnalyticsFilter) -> Result<soroban_sdk::String, ContractError> {
        let data = analytics::get_volume_chart(&env, filter)?;
        Ok(analytics::export_volume_csv(&env, &data))
    }

    /// Export user stats as CSV (no PII).
    pub fn export_user_stats_csv(env: Env, address: Address) -> soroban_sdk::String {
        let snapshot = analytics::get_user_stats(&env, &address);
        analytics::export_user_stats_csv(&env, &snapshot)
    }

    // Onboarding Flow
    // -------------------------------------------------------------------------

    /// Start (or resume) the onboarding flow for a user.
    /// Idempotent — safe to call multiple times; returns existing progress if
    /// already started. Returns AlreadyInitialized if onboarding is finished.
    pub fn start_onboarding(env: Env, user: Address) -> Result<OnboardingProgress, ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        onboarding::start_onboarding(&env, user)
    }

    /// Mark a specific onboarding step (0-based index) as completed.
    /// The step must currently be Pending; completing an already-done or
    /// skipped step returns InvalidStatus.
    pub fn complete_onboarding_step(env: Env, user: Address, step_index: u32) -> Result<OnboardingProgress, ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        onboarding::complete_step(&env, user, step_index)
    }

    /// Skip a specific onboarding step (0-based index).
    /// The step must currently be Pending. Skipping does not block progress.
    pub fn skip_onboarding_step(env: Env, user: Address, step_index: u32) -> Result<OnboardingProgress, ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        onboarding::skip_step(&env, user, step_index)
    }

    /// Exit onboarding entirely. All remaining Pending steps are marked
    /// Skipped and the flow is marked finished. No existing data is modified.
    pub fn exit_onboarding(env: Env, user: Address) -> Result<OnboardingProgress, ContractError> {
        if !is_initialized(&env) { return Err(ContractError::NotInitialized); }
        onboarding::exit_onboarding(&env, user)
    }

    /// Return the current onboarding progress for a user, or None if not started.
    pub fn get_onboarding_progress(env: Env, user: Address) -> Option<OnboardingProgress> {
        onboarding::get_progress(&env, &user)
    }

    /// Return true if the user has an active (started, not finished) onboarding flow.
    pub fn is_onboarding_active(env: Env, user: Address) -> bool {
        onboarding::is_onboarding_active(&env, &user)
    }

    // -------------------------------------------------------------------------
    // Audit Log
    // -------------------------------------------------------------------------

    /// Retrieve a single audit entry by its sequential ID.
    pub fn get_audit_log(env: Env, id: u64) -> Option<audit::AuditEntry> {
        audit::get_audit_log(&env, id)
    }

    /// Retrieve a page of audit entries in reverse-chronological order.
    /// Pass `from_id = 0` to start from the most recent entry.
    pub fn get_audit_logs(
        env: Env,
        from_id: u64,
        limit: u32,
    ) -> soroban_sdk::Vec<audit::AuditEntry> {
        audit::get_audit_logs(&env, from_id, limit)
    }

    /// Return the total number of audit entries recorded.
    pub fn audit_count(env: Env) -> u64 {
        audit::audit_count(&env)
    }
}
