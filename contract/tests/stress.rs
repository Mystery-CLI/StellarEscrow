//! Stress Testing Framework for StellarEscrow Contract
//!
//! Validates contract performance under load across five dimensions:
//!   1. Load scenarios  — high-volume trade creation, funding, completion
//!   2. Benchmarks      — per-operation instruction counts via `env.cost_estimate()`
//!   3. Automation      — parameterised helpers that drive all scenarios
//!   4. Resource usage  — CPU instructions and memory bytes tracked per scenario
//!   5. Performance limits — assertions that enforce documented maximums
//!
//! Run with:
//!   cargo test --test stress -- --nocapture

#![cfg(test)]

extern crate std;

use std::time::Instant;

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

use stellar_escrow_contract::{
    OptionalMetadata, StellarEscrowContract, StellarEscrowContractClient, TradeStatus,
};

// ---------------------------------------------------------------------------
// Performance limits (documented maximums)
// ---------------------------------------------------------------------------

/// Maximum acceptable Soroban CPU instructions for a single trade creation.
const MAX_INSTRUCTIONS_CREATE_TRADE: u64 = 25_000_000;
/// Maximum acceptable instructions for fund + complete + confirm (full happy path).
const MAX_INSTRUCTIONS_FULL_CYCLE: u64 = 75_000_000;
/// Maximum acceptable instructions for raising and resolving a dispute.
const MAX_INSTRUCTIONS_DISPUTE_CYCLE: u64 = 50_000_000;
/// Maximum number of trades that must be creatable in a single environment.
const LOAD_TRADE_COUNT: u32 = 100;
/// Maximum instructions for a batch of LOAD_TRADE_COUNT trade creations.
const MAX_INSTRUCTIONS_LOAD_BATCH: u64 = MAX_INSTRUCTIONS_CREATE_TRADE * LOAD_TRADE_COUNT as u64;

// ---------------------------------------------------------------------------
// Shared setup
// ---------------------------------------------------------------------------

struct Harness<'a> {
    env: Env,
    token_addr: Address,
    admin: Address,
    arbitrator: Address,
    client: StellarEscrowContractClient<'a>,
}

fn setup() -> Harness<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token_addr = sac.address();

    let contract_id = env.register_contract(None, StellarEscrowContract);
    let client = StellarEscrowContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &100u32); // 1 % fee
    client.register_arbitrator(&arbitrator);

    Harness {
        env,
        token_addr,
        admin,
        arbitrator,
        client,
    }
}

/// Mint `amount` USDC to `addr` and approve the contract to spend it.
fn fund_account(h: &Harness, addr: &Address, amount: i128) {
    token::StellarAssetClient::new(&h.env, &h.token_addr).mint(addr, &amount);
    token::Client::new(&h.env, &h.token_addr).approve(addr, &h.client.address, &amount, &200u32);
}

// ---------------------------------------------------------------------------
// Automation helpers
// ---------------------------------------------------------------------------

/// Create `n` trades and return their ids.
fn create_n_trades(h: &Harness, n: u32) -> std::vec::Vec<u64> {
    let mut ids = std::vec::Vec::with_capacity(n as usize);
    for _ in 0..n {
        let seller = Address::generate(&h.env);
        let buyer = Address::generate(&h.env);
        let id = h.client.create_trade(
            &seller,
            &buyer,
            &1_000_000u64,
            &None,
            &OptionalMetadata::None,
        );
        ids.push(id);
    }
    ids
}

/// Run a full happy-path cycle (create → fund → complete → confirm) for one trade.
/// Returns the trade id.
fn run_happy_path(h: &Harness) -> u64 {
    let seller = Address::generate(&h.env);
    let buyer = Address::generate(&h.env);
    fund_account(h, &buyer, 2_000_000);

    let id = h.client.create_trade(
        &seller,
        &buyer,
        &1_000_000u64,
        &None,
        &OptionalMetadata::None,
    );
    h.client.fund_trade(&id);
    h.client.complete_trade(&id);
    h.client.confirm_receipt(&id);
    id
}

/// Run a dispute cycle (create → fund → dispute → resolve) for one trade.
fn run_dispute_cycle(h: &Harness) {
    let seller = Address::generate(&h.env);
    let buyer = Address::generate(&h.env);
    fund_account(h, &buyer, 2_000_000);

    let id = h.client.create_trade(
        &seller,
        &buyer,
        &1_000_000u64,
        &Some(h.arbitrator.clone()),
        &OptionalMetadata::None,
    );
    h.client.fund_trade(&id);
    h.client.raise_dispute(&id);
    h.client.resolve_dispute(
        &id,
        &stellar_escrow_contract::DisputeResolution::ReleaseToSeller,
    );
}

// ---------------------------------------------------------------------------
// Benchmark helper — measures wall-clock time and reports instructions
// ---------------------------------------------------------------------------

struct BenchResult {
    wall_ns: u128,
    instructions: u64,
    mem_bytes: u64,
}

/// Execute `f`, capture Soroban resource usage, and return a `BenchResult`.
fn bench<F: FnOnce()>(env: &Env, f: F) -> BenchResult {
    let t0 = Instant::now();
    f();
    let wall_ns = t0.elapsed().as_nanos();

    let budget = env.cost_estimate().budget();
    let instructions = budget.cpu_instruction_cost().get_total_cpu_insns_consumed();
    let mem_bytes = budget.mem_bytes_cost().get_total_mem_bytes_consumed();

    BenchResult {
        wall_ns,
        instructions,
        mem_bytes,
    }
}

// ---------------------------------------------------------------------------
// 1. Load scenarios
// ---------------------------------------------------------------------------

#[test]
fn stress_load_create_100_trades() {
    let h = setup();
    let ids = create_n_trades(&h, LOAD_TRADE_COUNT);
    assert_eq!(ids.len(), LOAD_TRADE_COUNT as usize);
    // Verify last trade is retrievable and in Created state
    let last = h.client.get_trade(ids.last().unwrap());
    assert_eq!(last.status, TradeStatus::Created);
}

#[test]
fn stress_load_100_full_happy_paths() {
    let h = setup();
    for _ in 0..LOAD_TRADE_COUNT {
        run_happy_path(&h);
    }
    // All fees should have accumulated
    assert!(h.client.get_accumulated_fees() > 0);
}

#[test]
fn stress_load_50_dispute_cycles() {
    let h = setup();
    for _ in 0..50 {
        run_dispute_cycle(&h);
    }
}

#[test]
fn stress_load_mixed_concurrent_operations() {
    let h = setup();
    // Interleave creates, happy paths, and disputes
    for i in 0..30u32 {
        match i % 3 {
            0 => {
                create_n_trades(&h, 1);
            }
            1 => {
                run_happy_path(&h);
            }
            _ => {
                run_dispute_cycle(&h);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Performance benchmarks
// ---------------------------------------------------------------------------

#[test]
fn bench_create_trade_instructions() {
    let h = setup();
    let seller = Address::generate(&h.env);
    let buyer = Address::generate(&h.env);

    let r = bench(&h.env, || {
        h.client.create_trade(
            &seller,
            &buyer,
            &1_000_000u64,
            &None,
            &OptionalMetadata::None,
        );
    });

    std::println!(
        "[bench] create_trade: {} instructions, {} mem bytes, {}ns wall",
        r.instructions,
        r.mem_bytes,
        r.wall_ns
    );
    assert!(
        r.instructions <= MAX_INSTRUCTIONS_CREATE_TRADE,
        "create_trade exceeded instruction limit: {} > {}",
        r.instructions,
        MAX_INSTRUCTIONS_CREATE_TRADE
    );
}

#[test]
fn bench_full_happy_path_instructions() {
    let h = setup();
    let seller = Address::generate(&h.env);
    let buyer = Address::generate(&h.env);
    fund_account(&h, &buyer, 2_000_000);

    let r = bench(&h.env, || {
        let id = h.client.create_trade(
            &seller,
            &buyer,
            &1_000_000u64,
            &None,
            &OptionalMetadata::None,
        );
        h.client.fund_trade(&id);
        h.client.complete_trade(&id);
        h.client.confirm_receipt(&id);
    });

    std::println!(
        "[bench] full_happy_path: {} instructions, {} mem bytes, {}ns wall",
        r.instructions,
        r.mem_bytes,
        r.wall_ns
    );
    assert!(
        r.instructions <= MAX_INSTRUCTIONS_FULL_CYCLE,
        "full happy path exceeded instruction limit: {} > {}",
        r.instructions,
        MAX_INSTRUCTIONS_FULL_CYCLE
    );
}

#[test]
fn bench_dispute_cycle_instructions() {
    let h = setup();
    let seller = Address::generate(&h.env);
    let buyer = Address::generate(&h.env);
    fund_account(&h, &buyer, 2_000_000);

    let r = bench(&h.env, || {
        let id = h.client.create_trade(
            &seller,
            &buyer,
            &1_000_000u64,
            &Some(h.arbitrator.clone()),
            &OptionalMetadata::None,
        );
        h.client.fund_trade(&id);
        h.client.raise_dispute(&id);
        h.client.resolve_dispute(
            &id,
            &stellar_escrow_contract::DisputeResolution::ReleaseToBuyer,
        );
    });

    std::println!(
        "[bench] dispute_cycle: {} instructions, {} mem bytes, {}ns wall",
        r.instructions,
        r.mem_bytes,
        r.wall_ns
    );
    assert!(
        r.instructions <= MAX_INSTRUCTIONS_DISPUTE_CYCLE,
        "dispute cycle exceeded instruction limit: {} > {}",
        r.instructions,
        MAX_INSTRUCTIONS_DISPUTE_CYCLE
    );
}

#[test]
fn bench_fee_withdrawal_instructions() {
    let h = setup();
    // Generate some fees first
    run_happy_path(&h);

    let recipient = Address::generate(&h.env);
    let r = bench(&h.env, || {
        h.client.withdraw_fees(&recipient);
    });

    std::println!(
        "[bench] withdraw_fees: {} instructions, {} mem bytes, {}ns wall",
        r.instructions,
        r.mem_bytes,
        r.wall_ns
    );
}

// ---------------------------------------------------------------------------
// 3. Resource usage monitoring
// ---------------------------------------------------------------------------

#[test]
fn monitor_resource_usage_scales_linearly() {
    let h = setup();

    // Measure cost for 1 trade
    let seller1 = Address::generate(&h.env);
    let buyer1 = Address::generate(&h.env);
    let r1 = bench(&h.env, || {
        h.client.create_trade(
            &seller1,
            &buyer1,
            &1_000_000u64,
            &None,
            &OptionalMetadata::None,
        );
    });

    // Measure cost for 10 more trades (cumulative)
    let r10 = bench(&h.env, || {
        for _ in 0..10 {
            let s = Address::generate(&h.env);
            let b = Address::generate(&h.env);
            h.client
                .create_trade(&s, &b, &1_000_000u64, &None, &OptionalMetadata::None);
        }
    });

    std::println!(
        "[monitor] 1 trade: {} insns | 10 trades: {} insns | ratio: {:.2}x",
        r1.instructions,
        r10.instructions,
        r10.instructions as f64 / r1.instructions.max(1) as f64
    );

    // 10 trades should cost less than 15x a single trade (sub-linear overhead acceptable)
    assert!(
        r10.instructions <= r1.instructions * 15,
        "resource usage grew super-linearly: {} vs {} * 15",
        r10.instructions,
        r1.instructions
    );
}

#[test]
fn monitor_memory_usage_create_trade() {
    let h = setup();
    let seller = Address::generate(&h.env);
    let buyer = Address::generate(&h.env);

    let r = bench(&h.env, || {
        h.client.create_trade(
            &seller,
            &buyer,
            &1_000_000u64,
            &None,
            &OptionalMetadata::None,
        );
    });

    std::println!("[monitor] create_trade memory: {} bytes", r.mem_bytes);
    // Sanity: must use some memory but not an absurd amount (< 1 MB per op)
    assert!(
        r.mem_bytes < 1_000_000,
        "memory usage unexpectedly high: {}",
        r.mem_bytes
    );
}

// ---------------------------------------------------------------------------
// 4. Edge-case stress scenarios
// ---------------------------------------------------------------------------

#[test]
fn stress_max_amount_trade() {
    let h = setup();
    let seller = Address::generate(&h.env);
    let buyer = Address::generate(&h.env);
    let max_amount = u64::MAX / 2; // avoid overflow in fee calc
    fund_account(&h, &buyer, max_amount as i128);

    let id = h
        .client
        .create_trade(&seller, &buyer, &max_amount, &None, &OptionalMetadata::None);
    let trade = h.client.get_trade(&id);
    assert_eq!(trade.amount, max_amount);
}

#[test]
fn stress_rapid_fee_updates() {
    let h = setup();
    for bps in [0u32, 50, 100, 500, 1000, 9999, 10000] {
        h.client.update_fee(&bps);
        assert_eq!(h.client.get_platform_fee_bps(), bps);
    }
}

#[test]
fn stress_arbitrator_churn() {
    let h = setup();
    // Register and remove 20 arbitrators
    let arbs: std::vec::Vec<Address> = (0..20).map(|_| Address::generate(&h.env)).collect();
    for a in &arbs {
        h.client.register_arbitrator(a);
    }
    for a in &arbs {
        assert!(h.client.is_arbitrator_registered(a));
        h.client.remove_arbitrator_fn(a);
        assert!(!h.client.is_arbitrator_registered(a));
    }
}

// ---------------------------------------------------------------------------
// 5. Batch load benchmark — validates LOAD_TRADE_COUNT instruction budget
// ---------------------------------------------------------------------------

#[test]
fn bench_batch_create_100_trades_total_instructions() {
    let h = setup();

    let r = bench(&h.env, || {
        create_n_trades(&h, LOAD_TRADE_COUNT);
    });

    std::println!(
        "[bench] batch {} trades: {} total instructions, {} mem bytes, {}ns wall",
        LOAD_TRADE_COUNT,
        r.instructions,
        r.mem_bytes,
        r.wall_ns
    );
    assert!(
        r.instructions <= MAX_INSTRUCTIONS_LOAD_BATCH,
        "batch create exceeded instruction budget: {} > {}",
        r.instructions,
        MAX_INSTRUCTIONS_LOAD_BATCH
    );
}
