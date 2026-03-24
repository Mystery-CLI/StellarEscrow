#![no_std]

mod errors;
mod events;
mod storage;
mod templates;
mod tiers;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, token::TokenClient, Address, BytesN, Env};

use types::{METADATA_MAX_ENTRIES, METADATA_MAX_VALUE_LEN};

pub use errors::ContractError;
pub use types::{
    CrossChainInfo, DisputeResolution, MetadataEntry, OptionalMetadata, TierConfig,
    TemplateTerms, TemplateVersion, Trade, TradeMetadata, TradeStatus, TradeTemplate,
    UserTier, UserTierInfo,
};

use storage::{
    add_accumulated_fees, get_accumulated_fees, get_admin, get_fee_bps, get_trade,
    get_usdc_token, has_arbitrator, increment_trade_counter, is_initialized,
    is_paused, remove_arbitrator, save_arbitrator, save_trade, set_accumulated_fees, set_admin,
    set_fee_bps, set_initialized, set_paused, set_trade_counter, set_usdc_token,
    get_version, set_version,
    get_bridge_oracle, set_bridge_oracle, save_cross_chain_info, get_cross_chain_info,
};

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

fn validate_metadata(meta: &OptionalMetadata) -> Result<(), ContractError> {
    if let OptionalMetadata::Some(ref m) = meta {
        if m.entries.len() > METADATA_MAX_ENTRIES {
            return Err(ContractError::MetadataTooManyEntries);
        }
        for entry in m.entries.iter() {
            if entry.value.len() > METADATA_MAX_VALUE_LEN {
                return Err(ContractError::MetadataValueTooLong);
            }
        }
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

    /// Withdraw accumulated fees (admin only)
    pub fn withdraw_fees(env: Env, to: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        let fees = get_accumulated_fees(&env)?;
        if fees == 0 {
            return Err(ContractError::NoFeesToWithdraw);
        }
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &to, &(fees as i128));
        set_accumulated_fees(&env, 0);
        events::emit_fees_withdrawn(&env, fees, to);
        Ok(())
    }

    /// Create a new trade with optional metadata
    pub fn create_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        arbitrator: Option<Address>,
        metadata: OptionalMetadata,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        seller.require_auth();
        if let Some(ref arb) = arbitrator {
            if !has_arbitrator(&env, arb) {
                return Err(ContractError::ArbitratorNotRegistered);
            }
        }
        validate_metadata(&metadata)?;
        let trade_id = increment_trade_counter(&env)?;
        let fee = calc_fee(&env, &seller, amount)?;
        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee,
            arbitrator,
            status: TradeStatus::Created,
            metadata,
        };
        save_trade(&env, trade_id, &trade);
        events::emit_trade_created(&env, trade_id, seller, buyer, amount);
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
        Ok(())
    }

    /// Seller marks trade as completed
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
        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &trade.seller, &(payout as i128));
        // Single read-modify-write for fees
        add_accumulated_fees(&env, trade.fee)?;
        tiers::record_volume(&env, &trade.seller, trade.amount)?;
        tiers::record_volume(&env, &trade.buyer, trade.amount)?;
        events::emit_trade_confirmed(&env, trade_id, payout, trade.fee);
        Ok(())
    }

    /// Raise a dispute
    pub fn raise_dispute(env: Env, trade_id: u64, caller: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded && trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        if trade.arbitrator.is_none() {
            return Err(ContractError::ArbitratorNotRegistered);
        }
        if caller != trade.buyer && caller != trade.seller {
            return Err(ContractError::Unauthorized);
        }
        caller.require_auth();
        trade.status = TradeStatus::Disputed;
        save_trade(&env, trade_id, &trade);
        events::emit_dispute_raised(&env, trade_id, caller);
        Ok(())
    }

    /// Resolve a dispute (arbitrator only)
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
        let arbitrator = trade.arbitrator.ok_or(ContractError::ArbitratorNotRegistered)?;
        arbitrator.require_auth();
        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
        let recipient = match resolution {
            DisputeResolution::ReleaseToBuyer => trade.buyer.clone(),
            DisputeResolution::ReleaseToSeller => trade.seller.clone(),
        };
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &recipient, &(payout as i128));
        // Single read-modify-write for fees
        add_accumulated_fees(&env, trade.fee)?;
        events::emit_dispute_resolved(&env, trade_id, resolution, recipient);
        Ok(())
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
        Ok(())
    }

    /// Get trade details
    pub fn get_trade(env: Env, trade_id: u64) -> Result<Trade, ContractError> {
        get_trade(&env, trade_id)
    }

    /// Get accumulated fees
    pub fn get_accumulated_fees(env: Env) -> Result<u64, ContractError> {
        get_accumulated_fees(&env)
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
    // Emergency Pause
    // -------------------------------------------------------------------------

    /// Pause all contract operations (admin only).
    pub fn pause(env: Env) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_paused(&env, true);
        events::emit_paused(&env, admin);
        Ok(())
    }

    /// Unpause the contract (admin only).
    pub fn unpause(env: Env) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_paused(&env, false);
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
        let fee = calc_fee(&env, &seller, amount)?;

        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee,
            arbitrator: terms.default_arbitrator,
            status: TradeStatus::Created,
            metadata: terms.default_metadata,
        };

        save_trade(&env, trade_id, &trade);
        events::emit_trade_created(&env, trade_id, seller, buyer, amount);
        events::emit_trade_from_template(&env, trade_id, template_id, version);
        Ok(trade_id)
    }

    /// Get a template by ID.
    pub fn get_template(env: Env, template_id: u64) -> Result<TradeTemplate, ContractError> {
        storage::get_template(&env, template_id)
    }

    // -------------------------------------------------------------------------
    // Upgrade Mechanism
    // -------------------------------------------------------------------------

    /// Upgrade the contract WASM (admin only).
    /// After calling this, invoke `migrate()` if state changes are needed.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        events::emit_upgraded(&env, get_version(&env));
        Ok(())
    }

    /// Run post-upgrade state migration.
    /// `expected_version` must match the current stored version to prevent
    /// accidental double-application. Sets version to `expected_version + 1`.
    pub fn migrate(env: Env, expected_version: u32) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        let current = get_version(&env);
        if current != expected_version {
            return Err(ContractError::MigrationVersionMismatch);
        }
        // --- place version-specific migration logic here ---
        // e.g. if expected_version == 1 { backfill_new_field(&env); }
        let next = current.checked_add(1).ok_or(ContractError::Overflow)?;
        set_version(&env, next);
        events::emit_migrated(&env, current, next);
        Ok(())
    }

    /// Returns the current contract version.
    pub fn version(env: Env) -> u32 {
        get_version(&env)
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
            arbitrator,
            status: TradeStatus::AwaitingBridge,
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
}


