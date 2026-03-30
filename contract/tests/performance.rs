#![cfg(test)]

mod common;

use common::{approve_funding, create_trade, setup};
use std::time::Instant;
use soroban_sdk::testutils::Address as _;
use stellar_escrow_contract::{DisputeResolution, OptionalMetadata};

const MAX_LIFECYCLE_NS: u128 = 200_000_000;
const MAX_DISPUTE_WITH_INSURANCE_NS: u128 = 250_000_000;

struct BenchResult {
    wall_ns: u128,
}

fn bench<F: FnOnce()>(env: &soroban_sdk::Env, f: F) -> BenchResult {
    let start = Instant::now();
    f();
    let _ = env;
    BenchResult { wall_ns: start.elapsed().as_nanos() }
}

#[test]
fn perf_benchmark_standard_lifecycle() {
    let h = setup();

    let result = bench(&h.env, || {
        let id = create_trade(&h, 1_000_000);
        approve_funding(&h, 1_000_000);
        h.client.fund_trade(&id);
        h.client.complete_trade(&id);
        h.client.confirm_receipt(&id);
    });

    std::println!(
        "[bench] standard_lifecycle: {}ns wall",
        result.wall_ns
    );

    assert!(result.wall_ns <= MAX_LIFECYCLE_NS);
}

#[test]
fn perf_benchmark_dispute_and_insurance_path() {
    let h = setup();
    let provider = soroban_sdk::Address::generate(&h.env);
    h.client.register_insurance_provider(&provider);
    soroban_sdk::token::StellarAssetClient::new(&h.env, &h.token_addr)
        .mint(&provider, &10_000_000i128);

    let result = bench(&h.env, || {
        let id = h.client.create_trade(
            &h.seller,
            &h.buyer,
            &1_000_000u64,
            &Some(h.arbitrator.clone()),
            &OptionalMetadata::None,
        );
        approve_funding(&h, 1_000_000);
        h.client.fund_trade(&id);
        h.client.purchase_insurance(&id, &provider, &100u32, &250_000u64);
        h.client.raise_dispute(&id, &h.buyer);
        h.client
            .resolve_dispute(&id, &DisputeResolution::ReleaseToSeller);
    });

    std::println!(
        "[bench] dispute_insurance: {}ns wall",
        result.wall_ns
    );

    assert!(result.wall_ns <= MAX_DISPUTE_WITH_INSURANCE_NS);
}
