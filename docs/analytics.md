# Trade Analytics (Issue #20)

The `analytics` module (`contract/src/analytics.rs`) tracks escrow metrics on-chain and exposes them through a single query function.

## Metrics

### All-Time (`PlatformMetrics`)

| Field | Description |
|---|---|
| `total_volume` | Sum of all trade amounts ever created (stroops) |
| `trades_created` | Total trades ever created |
| `trades_funded` | Trades that reached the Funded state |
| `trades_completed` | Trades confirmed by the buyer (successful) |
| `trades_disputed` | Trades that entered dispute |
| `trades_cancelled` | Trades cancelled or expired |
| `total_fees_collected` | Cumulative platform fees (stroops) |

### Derived (`PlatformStats`)

| Field | Formula |
|---|---|
| `success_rate_bps` | `completed / (completed + cancelled + disputed) × 10 000` |
| `dispute_rate_bps` | `disputed / funded × 10 000` |
| `active_trades` | `created − completed − cancelled − disputed` |

Both rates are expressed in **basis points** (10 000 bps = 100 %).  
Returns `0` when the denominator is zero (no terminal trades yet).

### Time Windows (`WindowMetrics`)

Rolling windows reset automatically when the period elapses.

| Variant | Period |
|---|---|
| `Last24h` | 86 400 s |
| `Last7d` | 604 800 s |
| `Last30d` | 2 592 000 s |
| `AllTime` | Never resets |

Each window tracks: `volume`, `trades_created`, `trades_completed`, `trades_cancelled`, `trades_disputed`.

### Platform Usage

`unique_addresses` — count of distinct buyer/seller `Address` values that have ever appeared in a `create_trade` call. Stored as a persistent flag per address so duplicates are never double-counted.

## Query Function

```rust
pub fn analytics_query(env: Env, window: TimeWindow) -> AnalyticsResult
```

Returns an `AnalyticsResult`:

```rust
pub struct AnalyticsResult {
    pub all_time: PlatformStats,   // aggregate stats + derived rates
    pub window: WindowMetrics,     // metrics for the requested time window
    pub unique_addresses: u64,     // distinct buyer/seller addresses
}
```

### Soroban CLI example

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source dev \
  --network testnet \
  -- analytics_query \
  --window '{"AllTime": {}}'
```

For a 24-hour window:

```bash
  -- analytics_query --window '{"Last24h": {}}'
```

### JavaScript (stellar-sdk)

```ts
const result = await contract.call('analytics_query', { window: { Last7d: {} } });
console.log('Success rate:', result.all_time.success_rate_bps / 100, '%');
console.log('Unique addresses:', result.unique_addresses);
console.log('7-day volume:', result.window.volume);
```

## Update Hooks

The following hooks in `lib.rs` keep metrics current:

| Hook | Called from |
|---|---|
| `on_trade_created(env, amount, seller, buyer)` | `create_trade`, `create_multisig_trade` |
| `on_trade_funded(env)` | `fund_trade` |
| `on_trade_completed(env, fee)` | `confirm_receipt` |
| `on_trade_disputed(env)` | `raise_dispute` |
| `on_trade_cancelled(env)` | `cancel_trade`, `expire_bridge_trade` |
| `on_dispute_resolved(env, arb, resolution)` | `resolve_dispute` |
