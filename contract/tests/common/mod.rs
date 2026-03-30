#![allow(dead_code)]

extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

use stellar_escrow_contract::{
    types::{KycStatus, UserCompliance},
    OptionalMetadata, StellarEscrowContract, StellarEscrowContractClient,
};

pub struct Harness<'a> {
    pub env: Env,
    pub token_addr: Address,
    pub admin: Address,
    pub seller: Address,
    pub buyer: Address,
    pub arbitrator: Address,
    pub client: StellarEscrowContractClient<'a>,
}

pub fn setup() -> Harness<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token_addr = sac.address();

    token::StellarAssetClient::new(&env, &token_addr).mint(&buyer, &2_000_000_000i128);

    let contract_id = env.register_contract(None, StellarEscrowContract);
    let client = StellarEscrowContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &100u32);

    let compliant = UserCompliance {
        kyc_status: KycStatus::Verified,
        aml_cleared: true,
        jurisdiction: soroban_sdk::String::from_str(&env, "US"),
    };
    client.set_user_compliance(&admin, &seller, &compliant);
    client.set_user_compliance(&admin, &buyer, &compliant);
    client.set_user_compliance(&admin, &arbitrator, &compliant);
    client.register_arbitrator(&arbitrator);

    Harness {
        env,
        token_addr,
        admin,
        seller,
        buyer,
        arbitrator,
        client,
    }
}

pub fn approve_funding(h: &Harness, amount: i128) {
    token::Client::new(&h.env, &h.token_addr).approve(
        &h.buyer,
        &h.client.address,
        &amount,
        &200u32,
    );
}

pub fn create_trade(h: &Harness, amount: u64) -> u64 {
    h.client.create_trade(
        &h.seller,
        &h.buyer,
        &amount,
        &None,
        &OptionalMetadata::None,
    )
}

pub fn create_disputed_trade(h: &Harness, amount: u64) -> u64 {
    let id = h.client.create_trade(
        &h.seller,
        &h.buyer,
        &amount,
        &Some(h.arbitrator.clone()),
        &OptionalMetadata::None,
    );
    approve_funding(h, amount as i128);
    h.client.fund_trade(&id);
    h.client.raise_dispute(&id, &h.buyer);
    id
}

pub fn create_completed_trade(h: &Harness, amount: u64) -> u64 {
    let id = create_trade(h, amount);
    approve_funding(h, amount as i128);
    h.client.fund_trade(&id);
    h.client.complete_trade(&id);
    id
}

pub fn advance_ledger_sequence(h: &Harness, delta: u32) {
    h.env.ledger().with_mut(|ledger| {
        ledger.sequence_number += delta;
    });
}
