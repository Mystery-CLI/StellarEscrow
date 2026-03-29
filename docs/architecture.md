# Architecture

> StellarEscrow — System Design Reference  
> Audience: Engineers, DevOps, Technical Leadership

## Table of Contents

- [System Overview](#system-overview)
- [Component Inventory](#component-inventory)
- [Blockchain Layer — Soroban Contract](#blockchain-layer--soroban-contract)
- [Off-Chain Infrastructure](#off-chain-infrastructure)
- [CLI Flow](#cli-flow)
- [Networking & Security Model](#networking--security-model)
- [Observability Stack](#observability-stack)
- [Data Flow: Complete Trade Lifecycle](#data-flow-complete-trade-lifecycle)

---

## System Overview

StellarEscrow is a decentralized escrow platform built on the Stellar blockchain using Soroban smart contracts. It enables trustless peer-to-peer trades with USDC settlement, dispute arbitration, and tiered fee structures — all enforced by immutable on-chain logic.

> **Key Design Principle:** All financial logic lives in the Soroban smart contract. The off-chain stack (indexer, API, frontend) is stateless and fully reconstructible from blockchain event history at any time.

The platform is organized into four logical tiers:

| Tier | Components | Role |
|------|-----------|------|
| Blockchain | Soroban WASM contract | Escrow state machine, USDC transfers, arbitration |
| Indexer | Rust + Tokio service | Event polling, DB writes, WebSocket fanout |
| API | Node.js gateway | REST endpoints, auth, analytics |
| Presentation | SvelteKit + Nginx | Web UI, TLS termination, CDN headers |

---

## Component Inventory

| Component | Technology | Language | Port |
|-----------|-----------|----------|------|
| Smart Contract | Soroban / Stellar | Rust | On-chain |
| Indexer (`indexer/`) | Rust + Tokio | Rust | 3000 (internal) |
| API Gateway (`api/`) | Node.js | TypeScript | 4000 (internal) |
| Client (`client/`) | SvelteKit + Vite | TypeScript | 3001 |
| Nginx | nginx:alpine | — | 80, 443 |
| PostgreSQL 15 | postgres:15-alpine | SQL | 5432 (internal) |
| Redis 7 | redis:7-alpine | — | 6379 (internal) |
| Prometheus | prom/prometheus | — | 9090 (internal) |
| Grafana | grafana/grafana | — | 3002 (internal) |
| Loki + Promtail | grafana/loki | — | 3100 (internal) |
| Alertmanager | prom/alertmanager | — | 9093 (internal) |

---

## Blockchain Layer — Soroban Contract

Source: `contract/src/lib.rs`

### Contract Functions

| Function | Actor | Description |
|----------|-------|-------------|
| `create_trade(seller, amount, asset)` | Seller | Initializes escrow, emits `created` |
| `get_funding_preview(trade_id, buyer)` | Buyer | Returns balance, allowance, fee breakdown |
| `execute_fund(trade_id, buyer, preview)` | Buyer | Transfers USDC into escrow, emits `funded` |
| `confirm_receipt(trade_id)` | Buyer | Releases funds to seller, emits `completed` |
| `raise_dispute(trade_id, reason)` | Buyer/Seller | Locks funds, assigns arbitrator, emits `disputed` |
| `resolve_dispute(trade_id, winner)` | Arbitrator | Releases funds to winner, emits `resolved` |
| `analytics_query(params)` | Read-only | Returns on-chain metrics |

### Trade State Machine

```
CREATED → FUNDED → COMPLETED
                ↘
              DISPUTED → RESOLVED
```

Transitions are enforced by the contract and cannot be bypassed off-chain.

### Fee Tiers (`contract/src/tiers.rs`)

| Tier | Cumulative Volume | Fee Rate |
|------|------------------|----------|
| Standard | < $10,000 | 1.50% |
| Silver | $10,000 – $100,000 | 1.00% |
| Gold | $100,000 – $1,000,000 | 0.75% |
| Platinum | > $1,000,000 | 0.50% |

---

## Off-Chain Infrastructure

### Indexer (`indexer/`)

The Rust-based indexer polls the Stellar Horizon API every **5 seconds** (matching ledger close time), processes contract events, writes normalized records to PostgreSQL, and serves WebSocket subscriptions to the frontend.

Key configuration (`indexer/config.toml`):

```toml
[stellar]
poll_interval_seconds = 5
network = "testnet"   # or "mainnet"

[database]
max_connections = 10
min_connections = 2

[cache]
redis_url = "redis://:${REDIS_PASSWORD}@redis:6379"
events_ttl_secs = 10
```

### API Gateway (`api/`)

Node.js gateway exposing REST and WebSocket endpoints. Validates API keys (`API_KEYS` / `ADMIN_KEYS`), proxies analytics to the indexer, and serves trade history with CSV export.

### Redis Caching Strategy

| Endpoint Pattern | TTL | Rationale |
|-----------------|-----|-----------|
| `GET /events*` | 10s | Matches 5s Stellar ledger interval |
| `GET /search*` | 30s | Search results change infrequently |
| `GET /stats` | 60s | Aggregate queries are expensive |
| `POST /events/replay` | No cache | Mutating operation |

> **Fallback:** Redis unavailability degrades performance gracefully — all requests fall through to PostgreSQL without errors.

### PostgreSQL Schema (Key Tables)

| Table | Description |
|-------|-------------|
| `trades` | One row per escrow trade; mirrors contract state |
| `events` | Raw contract events from Stellar ledger |
| `analytics` | Pre-aggregated metrics snapshots |
| `arbitrators` | Registered dispute arbitrators |

Migrations are in `indexer/migrations/` and run automatically on indexer startup.

---

## CLI Flow

The `ui/` crate provides a terminal UI (Rust/Ratatui) for interacting with the escrow contract directly from the command line. The flow mirrors the web UI:

```
1. User selects action (create trade / fund / confirm / dispute)
2. CLI calls contract function via Soroban SDK
3. Transaction signed with local keypair
4. Result displayed; events polled from indexer for confirmation
```

Source files: `ui/src/main.rs`, `ui/src/app.rs`, `ui/src/form.rs`, `ui/src/ui.rs`

---

## Networking & Security Model

### Network Topology

Only Nginx (80/443) and the SvelteKit client (3001) are publicly reachable. All other services are bound to internal Docker networks only.

```
Internet
    │
    ▼
[Nginx :80/:443]  ← TLS termination
    │
    ├──► [Client :3001]    (SvelteKit UI)
    ├──► [Indexer :3000]   (internal only)
    └──► [API :4000]       (internal only)
              │
              ├──► [PostgreSQL :5432]  (internal only)
              └──► [Redis :6379]       (internal only)
```

> **Rule:** Database and cache ports (5432, 6379) are never published to external interfaces. See `docker-compose.yml` — these are commented out explicitly.

### Container Security Controls

| Control | Applied To | Effect |
|---------|-----------|--------|
| `no-new-privileges:true` | indexer, api | Prevents privilege escalation |
| `read_only: true` | indexer, api | Read-only filesystem; `/tmp` is tmpfs |
| `requirepass` | redis | Password required for all connections |
| Internal-only bind | postgres, redis | Not reachable from host network |

### TLS

Managed by Certbot (Let's Encrypt). Certificates stored in the `letsencrypt` Docker volume, mounted read-only into Nginx. Auto-renewed every 12 hours. Config: `ssl.nginx.conf`.

---

## Observability Stack

```
Indexer ──► Prometheus ──► Grafana (dashboards)
                    └────► Alertmanager ──► Slack / PagerDuty
Container logs ──► Promtail ──► Loki ──► Grafana (Explore)
```

### Key SLIs

| Metric | Target | Alert Threshold |
|--------|--------|----------------|
| TTFB | < 200ms | > 1000ms for 5+ min |
| DB query mean | < 10ms | > 50ms for 3+ min |
| Redis cache hit rate | > 80% | < 60% for 10+ min |
| Indexer memory | < 256MB | > 400MB |
| API error rate (5xx) | < 0.1% | > 1% for 2+ min |

Monitoring config: `monitoring/prometheus.yml`, `monitoring/alert_rules.yml`, `monitoring/alertmanager.yml`

---

## Data Flow: Complete Trade Lifecycle

```
1. Seller calls create_trade() on Soroban contract
   └─► Contract stores trade on-chain, emits `created` event

2. Indexer detects `created` event (within 5s poll)
   └─► Writes trade record to PostgreSQL `trades` table

3. Buyer's browser calls GET /trades/:id via API
   └─► Redis cache miss → API queries PostgreSQL → response cached (10s)

4. Buyer calls get_funding_preview() on contract (read-only)
   └─► Receives FundingPreview with balance, allowance, fee breakdown

5. Buyer approves USDC allowance → calls execute_fund()
   └─► Contract transfers USDC into escrow, emits `funded` event

6. Indexer picks up `funded` event
   └─► Updates DB status to FUNDED

7. Buyer confirms receipt → calls confirm_receipt()
   └─► Funds released to seller, emits `completed` event

8. Indexer updates trade to COMPLETED
   └─► Analytics aggregated → Grafana dashboard refreshed
```

---

*See also: [Deployment Guide](deployment.md) | [Operations](operations.md) | [Troubleshooting](runbooks/troubleshooting.md)*
