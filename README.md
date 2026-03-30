# StellarEscrow

Decentralized escrow on Stellar/Soroban Network.

## Features
✅ Trade creation form (preview + confirm)
✅ Buyer funding flow (USDC approval + tx confirmation) 
✅ Seller completion + buyer receipt confirmation
✅ Dispute resolution with registered arbitrators
✅ Tiered fees based on volume
✅ Trade history & CSV export
✅ **New: Buyer funding UI** (`client/`)

## Quick Start

### Backend (Soroban Contract + Indexer)
```bash
# Deploy contract (Soroban CLI)
soroban contract deploy --wasm target/*.wasm --source dev --network testnet

# Run indexer
cd indexer
cargo run
```

### Frontend (SvelteKit UI)
```bash
cd client
pnpm install
pnpm dev    # http://localhost:5173
```

**Test Funding**: Visit `/trades/1` → **Fund Trade** → Review preview → Approve USDC → Confirm

## Contract Functions
```
Key funding flow:
```
- `get_funding_preview(trade_id, buyer)` → FundingPreview (balance, allowance)
- `execute_fund(trade_id, buyer, preview)` → transfers USDC, emits `funded`

## Analytics

Query on-chain trade metrics (volume, success rate, unique addresses, time windows) via `analytics_query`. See [docs/analytics.md](docs/analytics.md) for full details.

## Architecture
```
- `contract/` → Soroban WASM contract
- `indexer/` → Event monitoring + API/WebSocket  
- `client/` → SvelteKit UI (funding interface #32)

## Development
```
pnpm dev    # Frontend
cargo test  # Backend tests
docker-compose up  # Indexer + DB
```

## Smart Contract Testing

The Soroban contract now has dedicated suites for edge cases, security scenarios, integration flows, stress tests, and performance benchmarks under `contract/tests/`.

```bash
npm run test:contract
npm run test:contract:edge
npm run test:contract:security
npm run test:contract:integration
npm run test:contract:performance
npm run test:contract:stress
npm run test:contract:coverage
```

## Test Automation Framework

A full test automation framework is available under `scripts/` and `testing/`:

- `npm run test:orchestrate` (runs unit, api, uat, a11y workflows sequentially)
- `npm run test:analytics` (aggregates orchestration/test suite results)
- `npm run test:maintain` (flaky/broken test maintenance rules)

Example local invocation:

```bash
# health-check before running unit regression against API
node scripts/orchestrate.js --suite unit --env ci
```

CI workflow is in `.github/workflows/test-framework.yml` with nightly schedules, PR comments, artifact upload, and report snapshot.


