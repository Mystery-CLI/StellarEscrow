# 🏗️ Architecture & System Design

> **StellarEscrow** — Decentralized escrow on Stellar/Soroban Network  
> Version: 1.0 | Audience: Engineers, DevOps, Technical Leadership

---

## Table of Contents

- [System Overview](#1-system-overview)
- [Blockchain Layer — Soroban Contract](#2-blockchain-layer--soroban-contract)
- [Off-Chain Infrastructure](#3-off-chain-infrastructure)
- [Networking & Security](#4-networking--security)
- [Observability Stack](#5-observability-stack)
- [Deployment Environments](#6-deployment-environments)
- [Data Flow: Complete Trade Lifecycle](#7-data-flow-complete-trade-lifecycle)

---

## 1. System Overview

StellarEscrow is a decentralized escrow platform built on the Stellar blockchain using Soroban smart contracts. It enables trustless peer-to-peer trades with USDC settlement, dispute arbitration, and tiered fee structures — all enforced by immutable on-chain logic.

> **Key Design Principle:** All financial logic lives in the Soroban smart contract (immutable, on-chain). The off-chain stack (indexer, API, frontend) is stateless and reconstructible from blockchain event history at any time.

### 1.1 High-Level Architecture

The platform is organized into four logical tiers:

| Tier | Components | Role |
|------|-----------|------|
| **Blockchain** | Soroban WASM contract | Escrow state machine, USDC transfers, arbitration |
| **Indexer** | Rust + Tokio service | Event polling, DB writes, WebSocket fanout |
| **API** | Node.js gateway | REST endpoints, auth, analytics |
| **Presentation** | SvelteKit + Nginx | Web UI, TLS termination, CDN headers |

### 1.2 Component Inventory

| Component | Technology | Language | Role | Port |
|-----------|-----------|----------|------|------|
| Smart Contract | Soroban / Stellar | Rust | Escrow state machine, USDC settlement | On-chain |
| Indexer | Rust + Tokio | Rust | Event polling, DB writes, WebSocket fanout | 3000 (internal) |
| API Gateway | Node.js | TypeScript | REST endpoints, auth, analytics | 4000 (internal) |
| Client (SvelteKit) | SvelteKit + Vite | TypeScript | Web UI — trade lifecycle, funding | 3001 |
| Nginx | nginx:alpine | — | TLS termination, reverse proxy, CDN headers | 80, 443 |
| PostgreSQL 15 | postgres:15-alpine | SQL | Indexed event & trade storage | 5432 (internal) |
| Redis 7 | redis:7-alpine | — | API response caching, rate-limit counters | 6379 (internal) |
| Prometheus | prom/prometheus | — | Metrics scraping & time-series storage | 9090 (internal) |
| Grafana | grafana/grafana | — | Dashboards & alert visualization | 3002 (internal) |
| Loki + Promtail | grafana/loki | — | Centralized log aggregation | 3100 (internal) |
| Alertmanager | prom/alertmanager | — | Alert routing (Slack, PagerDuty) | 9093 (internal) |
| Certbot | certbot/certbot | — | Automated Let's Encrypt TLS renewal | — |
| Backup Service | postgres:15-alpine | Bash | Daily pg_dump to S3 with retention | — |

---

## 2. Blockchain Layer — Soroban Contract

### 2.1 Contract Architecture

The escrow contract is the authoritative source of truth for all trade state. Deployed as a WASM binary to the Stellar Soroban environment, it is **immutable after deployment** — upgrades require migrating to a new contract address.

#### Core Contract Functions

| Function | Actor | Description |
|----------|-------|-------------|
| `create_trade(seller, amount, asset)` | Seller | Initializes escrow, sets terms, emits `created` event |
| `get_funding_preview(trade_id, buyer)` | Buyer | Returns `FundingPreview`: balance, allowance, fee breakdown |
| `execute_fund(trade_id, buyer, preview)` | Buyer | Transfers USDC into escrow, emits `funded` event |
| `confirm_receipt(trade_id)` | Buyer | Releases escrowed funds to seller, emits `completed` event |
| `raise_dispute(trade_id, reason)` | Buyer/Seller | Locks funds, assigns arbitrator, emits `disputed` event |
| `resolve_dispute(trade_id, winner)` | Arbitrator | Releases funds to winner, emits `resolved` event |
| `analytics_query(params)` | Read-only | Returns on-chain metrics: volume, success rate, unique addresses |

#### Trade State Machine

Every trade progresses through the following states. Transitions are enforced by the contract and cannot be bypassed:

```
CREATED → FUNDED → COMPLETED
                ↘
              DISPUTED → RESOLVED
```

| State | Description |
|-------|-------------|
| `CREATED` | Seller has initialized the trade; awaiting buyer funding |
| `FUNDED` | Buyer has deposited USDC; funds held in escrow |
| `COMPLETED` | Buyer confirmed receipt; funds released to seller |
| `DISPUTED` | One party raised a dispute; funds locked pending arbitration |
| `RESOLVED` | Arbitrator decision executed; trade closed |

### 2.2 Fee Structure

StellarEscrow uses tiered fees based on cumulative trade volume per address:

| Tier | Cumulative Volume | Fee Rate |
|------|------------------|----------|
| Standard | < $10,000 | 1.50% |
| Silver | $10,000 – $100,000 | 1.00% |
| Gold | $100,000 – $1,000,000 | 0.75% |
| Platinum | > $1,000,000 | 0.50% |

---

## 3. Off-Chain Infrastructure

### 3.1 Indexer Service

The Rust-based indexer bridges the Stellar blockchain to PostgreSQL. It polls the Stellar Horizon API at configurable intervals (default **5 seconds**, matching ledger close time), processes contract events, and writes normalized records to the database. It also serves as a WebSocket server for real-time frontend subscriptions.

#### Indexer Configuration (`config.toml`)

```toml
[database]
max_connections = 10    # Pool ceiling; increase for high-concurrency
min_connections = 2     # Warm connections always available

[stellar]
poll_interval_seconds = 5   # Matches Stellar ledger close time
network = "mainnet"          # testnet | mainnet

[cache]
redis_url = "redis://:${REDIS_PASSWORD}@redis:6379"
```

#### Redis Caching Strategy

| Endpoint Pattern | TTL | Rationale |
|-----------------|-----|-----------|
| `GET /events*` | 10s | High-frequency reads; matches 5s Stellar ledger interval |
| `GET /search*` | 30s | Search results change infrequently |
| `GET /stats` | 60s | Aggregate queries are expensive; staleness acceptable |
| `POST /events/replay` | No cache | Mutating operation — always hits database |

> **Fallback:** If Redis is unavailable, all requests fall through to PostgreSQL with no errors. Cache misses degrade performance gracefully rather than causing failures.

### 3.2 API Gateway

The Node.js API gateway exposes REST and WebSocket endpoints to the SvelteKit client and authenticated third-party consumers. It validates API keys (`API_KEYS` for standard access, `ADMIN_KEYS` for administrative endpoints), proxies analytics queries to the indexer, and serves trade history with CSV export.

#### Key Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string (DSN format) |
| `INDEXER_URL` | Yes | Internal URL of the indexer (`http://indexer:3000`) |
| `API_KEYS` | Yes | Comma-separated list of valid API keys |
| `ADMIN_KEYS` | Yes | Admin-level API keys for privileged endpoints |
| `NODE_ENV` | Yes | Must be `"production"` in production deployments |
| `REDIS_URL` | No | Optional; enables response caching if set |

### 3.3 Data Layer

#### PostgreSQL Schema (Key Tables)

| Table | Description | Indexes |
|-------|-------------|---------|
| `trades` | One row per escrow trade; mirrors contract state | trade_id (PK), status, buyer, seller, created_at |
| `events` | Raw contract events from Stellar ledger | event_type, trade_id, ledger_sequence, timestamp |
| `analytics` | Pre-aggregated metrics snapshots | time_window, metric_name |
| `arbitrators` | Registered dispute arbitrators | address (PK), active flag |

#### Connection Pool Sizing

> **Rule of thumb:** `max_connections = (2 × CPU cores) + effective_spindle_count`  
> For a 4-core host: set `max_connections = 10–15`

---

## 4. Networking & Security

### 4.1 Network Topology

Internal services (PostgreSQL, Redis, Prometheus, Grafana, Loki) are **bound to `127.0.0.1` only** — not reachable from outside the Docker network. Only Nginx (ports 80/443) and the client (port 3001) are publicly exposed.

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

> ⚠️ **Security Rule:** Database and cache ports (5432, 6379) are **NEVER** published to external interfaces. The `docker-compose.yml` comments these out explicitly.

### 4.2 TLS / SSL

TLS is managed by Certbot (Let's Encrypt) with automatic renewal every 12 hours. Certificates are stored in the `letsencrypt` Docker volume and mounted read-only into Nginx.

```bash
# Certificate renewal (runs every 12 hours via docker entrypoint)
certbot renew --webroot -w /var/www/certbot --quiet

# Cron-based renewal fallback
0 0,12 * * * /opt/stellarescrow/scripts/renew-certs.sh
```

### 4.3 Container Security

| Security Control | Applied To | Effect |
|-----------------|-----------|--------|
| `no-new-privileges:true` | indexer, api | Prevents privilege escalation via setuid binaries |
| `read_only: true` | indexer, api | Container filesystem is read-only; `/tmp` is tmpfs |
| `requirepass` (Redis) | redis | `REDIS_PASSWORD` required for all connections |
| Internal-only bind | postgres, redis | Not reachable from host network |
| `POSTGRES_PASSWORD` env | postgres | Mandatory via `?`-syntax — startup fails without it |

---

## 5. Observability Stack

### 5.1 Metrics Pipeline

```
Indexer ──► Prometheus ──► Grafana (dashboards)
                    └────► Alertmanager ──► Slack / PagerDuty
```

#### Key Performance Indicators

| Metric | Target | Alert Threshold |
|--------|--------|----------------|
| TTFB | < 200ms (good), < 800ms (acceptable) | > 1000ms for 5+ minutes |
| Largest Contentful Paint | < 2.5s | > 4.0s |
| DB query mean (hot paths) | < 10ms | > 50ms for 3+ minutes |
| Redis cache hit rate | > 80% | < 60% for 10+ minutes |
| Indexer memory usage | < 256MB | > 400MB |
| API error rate (5xx) | < 0.1% | > 1% for 2+ minutes |

### 5.2 Log Aggregation

Promtail ships container and host logs to Loki. Logs are queryable via Grafana's Explore interface using LogQL. All services use structured JSON logging with `RUST_LOG=info` for Rust services.

### 5.3 Web Vitals

The frontend collects Core Web Vitals (TTFB, LCP, FID, CLS) and CDN performance metrics via `observeWebVitals()` and `observeCdnPerformance()`. These beacon to `POST /api/metrics` and appear in Grafana alongside infrastructure metrics.

---

## 6. Deployment Environments

| Environment | Network | Contract | Purpose |
|-------------|---------|----------|---------|
| Development | Stellar Testnet | Dev contract (redeployable) | Local dev with `pnpm dev` / `cargo test` |
| Staging | Stellar Testnet | Staging contract | Pre-production validation; mirrors production config |
| Production | Stellar Mainnet | Immutable mainnet contract | Live traffic; RTO ≤ 2h, RPO ≤ 24h |

Each environment has its own `.env` file (`.env.development`, `.env.staging`, `.env.production`) with environment-specific secrets and RPC endpoints.

---

## 7. Data Flow: Complete Trade Lifecycle

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

*© 2026 StellarEscrow*
