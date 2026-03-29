# Stress Testing Framework

Validates StellarEscrow contract performance under load using Soroban's built-in cost estimation.

## Running

```bash
cd contract

# All stress tests with output
cargo test --test stress -- --nocapture

# Single scenario
cargo test --test stress bench_full_happy_path -- --nocapture
```

## Test Categories

| Category | Tests | What it validates |
|---|---|---|
| Load scenarios | `stress_load_*` | Contract handles 100 concurrent trades without error |
| Benchmarks | `bench_*` | Per-operation instruction counts stay within limits |
| Resource monitoring | `monitor_*` | Memory and CPU scale sub-linearly with trade count |
| Edge cases | `stress_max_amount_*`, `stress_rapid_*`, `stress_arbitrator_*` | Boundary conditions don't cause overflows or panics |

## Performance Limits

These are enforced as `assert!` guards in the test suite. Exceeding them fails CI.

| Operation | CPU Instruction Limit |
|---|---|
| `create_trade` | 25,000,000 |
| Full happy path (create → fund → complete → confirm) | 75,000,000 |
| Dispute cycle (create → fund → dispute → resolve) | 50,000,000 |
| Batch of 100 `create_trade` calls | 2,500,000,000 |

> Limits are based on Soroban's mainnet instruction budget of ~100M per transaction.
> The full happy path is designed to fit within a single transaction.

## Metrics Reported

Each `bench_*` test prints to stdout:

```
[bench] create_trade: 1234567 instructions, 98765 mem bytes, 4200000ns wall
```

- **instructions** — Soroban CPU instruction units consumed
- **mem bytes** — heap memory allocated during execution
- **wall** — real elapsed time (useful for local profiling, not enforced)

## Adding New Scenarios

1. Add a `#[test]` function in `contract/tests/stress.rs`
2. Use `bench(&h.env, || { ... })` to capture resource usage
3. Add an `assert!` against the appropriate `MAX_INSTRUCTIONS_*` constant
4. Document the new limit in the table above
