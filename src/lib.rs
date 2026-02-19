#![no_std]

mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};

pub use errors::ContractError;
pub use types::{DisputeResolution, Trade, TradeStatus};

use storage::{
    get_accumulated_fees, get_admin, get_fee_bps, get_trade, get_trade_counter, get_usdc_token,
    has_arbitrator, has_initialized, increment_trade_counter, is_initialized, remove_arbitrator,
    save_arbitrator, save_trade, set_accumulated_fees, set_admin, set_fee_bps, set_initialized,
    set_trade_counter, set_usdc_token,
};

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

        Ok(())
    }

    /// Register an arbitrator (admin only)
    pub fn register_arbitrator(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

        let admin = get_admin(&env)?;
        admin.require_auth();

        save_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_registered(&env, arbitrator);

        Ok(())
    }

    /// Remove an arbitrator (admin only)
    pub fn remove_arbitrator_fn(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

        let admin = get_admin(&env)?;
        admin.require_auth();

        remove_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_removed(&env, arbitrator);

        Ok(())
    }

    /// Update platform fee (admin only)
    pub fn update_fee(env: Env, fee_bps: u32) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

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
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

        let admin = get_admin(&env)?;
        admin.require_auth();

        let fees = get_accumulated_fees(&env)?;
        if fees == 0 {
            return Err(ContractError::NoFeesToWithdraw);
        }

        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);

        token_client.transfer(&env.current_contract_address(), &to, &(fees as i128));

        set_accumulated_fees(&env, 0);
        events::emit_fees_withdrawn(&env, fees, to);

        Ok(())
    }

    /// Create a new trade
    pub fn create_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        arbitrator: Option<Address>,
    ) -> Result<u64, ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }

        seller.require_auth();

        if let Some(ref arb) = arbitrator {
            if !has_arbitrator(&env, arb) {
                return Err(ContractError::ArbitratorNotRegistered);
            }
        }

        let trade_id = increment_trade_counter(&env)?;
        let fee_bps = get_fee_bps(&env)?;
        let fee = amount
            .checked_mul(fee_bps as u64)
            .ok_or(ContractError::Overflow)?
            .checked_div(10000)
            .ok_or(ContractError::Overflow)?;

        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee,
            arbitrator,
            status: TradeStatus::Created,
        };

        save_trade(&env, trade_id, &trade);
        events::emit_trade_created(&env, trade_id, seller, buyer, amount);

        Ok(trade_id)
    }

    /// Buyer funds the trade
    pub fn fund_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

        let mut trade = get_trade(&env, trade_id)?;

        if trade.status != TradeStatus::Created {
            return Err(ContractError::InvalidStatus);
        }

        trade.buyer.require_auth();

        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);

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
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

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
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

        let trade = get_trade(&env, trade_id)?;

        if trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }

        trade.buyer.require_auth();

        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);

        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;

        token_client.transfer(
            &env.current_contract_address(),
            &trade.seller,
            &(payout as i128),
        );

        let current_fees = get_accumulated_fees(&env)?;
        let new_fees = current_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?;
        set_accumulated_fees(&env, new_fees);

        events::emit_trade_confirmed(&env, trade_id, payout, trade.fee);

        Ok(())
    }

    /// Raise a dispute
    pub fn raise_dispute(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

        let mut trade = get_trade(&env, trade_id)?;

        if trade.status != TradeStatus::Funded && trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }

        if trade.arbitrator.is_none() {
            return Err(ContractError::ArbitratorNotRegistered);
        }

        let caller = env.invoker();
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
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

        let trade = get_trade(&env, trade_id)?;

        if trade.status != TradeStatus::Disputed {
            return Err(ContractError::InvalidStatus);
        }

        let arbitrator = trade.arbitrator.ok_or(ContractError::ArbitratorNotRegistered)?;
        arbitrator.require_auth();

        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);

        let recipient = match resolution {
            DisputeResolution::ReleaseToBuyer => trade.buyer.clone(),
            DisputeResolution::ReleaseToSeller => trade.seller.clone(),
        };

        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;

        token_client.transfer(
            &env.current_contract_address(),
            &recipient,
            &(payout as i128),
        );

        let current_fees = get_accumulated_fees(&env)?;
        let new_fees = current_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?;
        set_accumulated_fees(&env, new_fees);

        events::emit_dispute_resolved(&env, trade_id, resolution, recipient);

        Ok(())
    }

    /// Cancel an unfunded trade
    pub fn cancel_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

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
}

mod token {
    soroban_sdk::contractimport!(file = "./token.wasm");
}
