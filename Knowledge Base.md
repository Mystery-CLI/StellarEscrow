# 📚 Knowledge Base

> **StellarEscrow** — FAQs, Concepts & Developer Reference  
> Version: 1.0 | Audience: All Engineering Team Members  
> This is a **living document** — update it with every major change.

---

## Table of Contents

- [Platform Concepts](#1-platform-concepts)
- [Frequently Asked Questions](#2-frequently-asked-questions)
  - [Infrastructure & Operations](#21-infrastructure--operations)
  - [Development](#22-development)
- [Common Configuration Patterns](#3-common-configuration-patterns)
- [Monitoring Reference](#4-monitoring-reference)
- [Useful Scripts Reference](#5-useful-scripts-reference)
- [External Dependencies & Status Pages](#6-external-dependencies--status-pages)
- [Decision Log](#7-decision-log)

---

## 1. Platform Concepts

### 1.1 What is StellarEscrow?

StellarEscrow is a decentralized escrow protocol built on Stellar's Soroban smart contract platform. It enables trustless peer-to-peer trades where funds are held by the smart contract — not by any company or intermediary — and released only when both parties agree.

### 1.2 Why Stellar / Soroban?

| Property | Stellar Advantage |
|----------|------------------|
| Transaction finality | 3–5 second ledger close time (vs 12+ seconds on Ethereum) |
| Transaction cost | ~$0.00001 per operation (negligible for users) |
| USDC support | Native USDC on Stellar — no bridging required |
| Smart contracts | Soroban: WebAssembly-based, Rust-native, auditable |
| Regulatory clarity | Stellar Development Foundation (SDF) provides regulatory engagement |

### 1.3 Key Terminology

| Term | Definition |
|------|-----------|
| **Soroban** | Stellar's smart contract platform; contracts compile to WebAssembly (WASM) and run in a deterministic VM |
| **Ledger** | The unit of consensus on Stellar; a new ledger closes every ~5 seconds, analogous to a block on other chains |
| **Trade** | An escrow agreement between a buyer and seller, represented by a unique `trade_id` on-chain |
| **FundingPreview** | A struct returned by `get_funding_preview()` showing buyer balance, USDC allowance, and fee breakdown before committing funds |
| **Arbitrator** | A registered address authorized to resolve disputes; arbitrators are registered in the contract by the admin |
| **Indexer** | The off-chain Rust service that reads Stellar events and writes normalized data to PostgreSQL |
| **Horizon API** | Stellar's public REST API for querying ledger state; the indexer polls Horizon for contract events |
| **USDC** | USD Coin; the settlement currency for all StellarEscrow trades |
| **WASM** | WebAssembly; the binary format that Soroban smart contracts are compiled to |
| **XLM** | Stellar Lumens; used to pay transaction fees (separate from USDC trade amounts) |

---

## 2. Frequently Asked Questions

### 2.1 Infrastructure & Operations

---

#### Q: Why is the indexer not picking up new events?

Most likely causes in order of frequency:

1. **Stellar network incident** — check https://status.stellar.org
2. **PostgreSQL connection failure** — run [RB-001](03-TROUBLESHOOTING.md#rb-001-postgresql-failure)
3. **Redis OOM kill preventing startup** — `docker compose ps` and check exit codes (137 = OOM)
4. **Environment variable misconfiguration** — `docker compose config indexer`

---

#### Q: How do I add a new arbitrator?

Arbitrators are registered directly on the Soroban contract by the admin address:

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network mainnet \
  -- register_arbitrator \
  --arbitrator G<ADDRESS>
```

---

#### Q: Can I upgrade the smart contract?

**No.** Soroban contracts are immutable after deployment by design. If breaking changes are needed:
1. Deploy a new contract
2. Update `CONTRACT_ID` in `.env`
3. Migrate users to the new contract

Existing trades on the old contract continue to be served by the indexer indefinitely.

---

#### Q: What happens if Redis goes down?

The system degrades gracefully — all requests fall through to PostgreSQL. Performance will be degraded (higher latency, more DB load) but **no errors will occur**. Restart Redis and the cache will warm up automatically within a few minutes of traffic. See [RB-005](03-TROUBLESHOOTING.md#rb-005-redis-cache-failure).

---

#### Q: How do I check the current indexer lag?

```bash
# Check the health endpoint
curl -s http://localhost:3000/health | python3 -m json.tool
# Look for "ledger_lag" or "last_processed_ledger" field

# Check current Stellar ledger for comparison
curl -s https://horizon.stellar.org | python3 -m json.tool | grep "history_latest_ledger"
```

---

#### Q: How do I export trade history?

The API supports CSV export for trade history. Authenticated consumers can call:

```bash
curl -H "Authorization: Bearer $API_KEY" \
  "https://stellarescrow.app/api/trades?format=csv&from=2026-01-01&to=2026-03-31" \
  -o trades-export.csv
```

---

#### Q: What are the fee tiers and how are they calculated?

Fees are calculated **on-chain** by the contract based on cumulative trade volume per address:

| Tier | Volume | Rate |
|------|--------|------|
| Standard | < $10K | 1.50% |
| Silver | $10K – $100K | 1.00% |
| Gold | $100K – $1M | 0.75% |
| Platinum | > $1M | 0.50% |

Call `get_funding_preview()` to see the exact fee for any trade before committing.

---

#### Q: How do I manually trigger a backup?

```bash
# Via Docker (recommended)
docker compose run --rm backup bash /scripts/backup.sh

# Directly on host
DATABASE_URL=<url> S3_BUCKET=s3://stellarescrow-backups ./scripts/backup.sh
```

---

#### Q: How do I check if the last backup succeeded?

```bash
# Check backup log
tail -20 /var/log/stellar-backup.log

# List recent S3 backups
aws s3 ls s3://stellarescrow-backups/ | sort | tail -5
```

---

### 2.2 Development

---

#### Q: How do I set up a local development environment?

```bash
# 1. Install dependencies
cargo install --locked soroban-cli
npm install -g pnpm

# 2. Start backend services
docker compose up -d postgres redis

# 3. Run the indexer
cd indexer && cargo run

# 4. Start the frontend
cd client && pnpm install && pnpm dev
# Frontend available at http://localhost:5173
```

---

#### Q: How do I run the tests?

```bash
# Rust (contract + indexer)
cargo test

# Frontend unit tests
cd client && pnpm test

# End-to-end tests (requires running services)
pnpm exec playwright test
```

---

#### Q: How do I simulate a trade in development?

```bash
# Deploy to testnet first
soroban contract deploy \
  --wasm target/*.wasm \
  --source dev \
  --network testnet

# Visit http://localhost:5173/trades/1
# Click "Fund Trade" → Review Preview → Approve USDC → Confirm
```

---

#### Q: How do I replay events to backfill a gap?

```bash
curl -X POST http://localhost:3000/events/replay \
  -H "Authorization: Bearer $ADMIN_KEY" \
  -d '{"from_ledger": <START_LEDGER>, "to_ledger": <END_LEDGER>}'
```

---

## 3. Common Configuration Patterns

### 3.1 Nginx Configuration Quick Reference

| Config File | Purpose | Key Settings |
|-------------|---------|-------------|
| `ssl.nginx.conf` | TLS termination and HTTPS redirect | Let's Encrypt cert paths, HSTS, TLS 1.2/1.3 only |
| `cdn-headers.nginx.conf` | Cache-Control headers for CDN | Hashed assets: immutable 1yr; HTML: 5min; Service worker: no-store |
| `performance.nginx.conf` | Performance tuning | `worker_processes auto`; `worker_connections 4096`; gzip on; keepalive 32 |

#### CDN Cache Header Reference

| Asset Type | Cache-Control |
|------------|---------------|
| Hashed JS/CSS | `immutable, max-age=31536000` |
| Images/fonts | `max-age=86400, stale-while-revalidate=3600` |
| HTML/JSON | `max-age=300` |
| Service worker | `no-store` |

### 3.2 Indexer `config.toml` Full Reference

```toml
[database]
max_connections = 10      # Pool ceiling
min_connections = 2       # Warm connections always ready

[stellar]
poll_interval_seconds = 5  # Match Stellar ledger close time
network = "mainnet"        # "testnet" | "mainnet"
contract_id = "<CONTRACT>" # Soroban contract address

[cache]
redis_url = "redis://:${REDIS_PASSWORD}@redis:6379"
# TTL values (in seconds)
events_ttl = 10
search_ttl = 30
stats_ttl  = 60

[server]
host = "0.0.0.0"
port = 3000
```

### 3.3 Docker Resource Limits (Production)

Add to each service in `docker-compose.yml` for production resource governance:

```yaml
deploy:
  resources:
    limits:
      cpus: '1.0'
      memory: 512M
    reservations:
      cpus: '0.25'
      memory: 128M
```

---

## 4. Monitoring Reference

### 4.1 Prometheus Metrics Reference

| Metric Name | Type | Description |
|-------------|------|-------------|
| `indexer_ledger_lag_seconds` | Gauge | Seconds behind current Stellar ledger |
| `indexer_events_processed_total` | Counter | Total contract events processed since startup |
| `db_query_duration_seconds` | Histogram | Database query latency by query type |
| `api_request_duration_seconds` | Histogram | API response time by endpoint and method |
| `api_requests_total` | Counter | Total API requests by status code |
| `redis_hit_rate` | Gauge | Redis cache hit ratio (0.0 – 1.0) |
| `backup_success` | Gauge | Last backup status: `1` = success, `0` = failure |
| `dr_test_result` | Gauge | Last DR test result: `1` = pass, `0` = fail |
| `trade_volume_usdc_total` | Counter | Cumulative USDC volume processed |
| `active_trades_total` | Gauge | Current number of trades in `FUNDED` state |

### 4.2 LogQL Quick Reference (Grafana Loki)

Use these in Grafana → Explore to diagnose issues:

```logql
# All indexer errors in the last hour
{container="indexer"} |= "ERROR" | json | line_format "{{.timestamp}} {{.message}}"

# API 5xx errors
{container="api"} | json | status >= 500

# Slow database queries (> 100ms)
{container="indexer"} |= "slow_query" | json | duration > 100

# Backup results
{container="backup"} |= "backup_success|backup_failed"
```

---

## 5. Useful Scripts Reference

| Script | Purpose | Usage |
|--------|---------|-------|
| `scripts/backup.sh` | Manual database backup to S3 | `DATABASE_URL=<url> S3_BUCKET=<bucket> ./scripts/backup.sh` |
| `scripts/restore.sh` | Restore database from backup file or S3 | `./scripts/restore.sh <path-or-s3-uri>` |
| `scripts/dr-test.sh` | Run disaster recovery validation test | `./scripts/dr-test.sh [backup-path]` |
| `scripts/perf-analyze.sh` | Performance bottleneck analysis | `./scripts/perf-analyze.sh [--json]` |
| `scripts/renew-certs.sh` | TLS certificate renewal check | `./scripts/renew-certs.sh` |

---

## 6. External Dependencies & Status Pages

| Service | Purpose | Status Page |
|---------|---------|-------------|
| Stellar Network | Blockchain layer; all contract calls depend on this | https://status.stellar.org |
| Stellar Horizon API | REST API for ledger queries used by the indexer | https://status.stellar.org |
| Let's Encrypt | TLS certificates | https://letsencrypt.status.io |
| AWS S3 | Database backup storage | https://health.aws.amazon.com |
| Docker Hub | Base images (postgres, redis, nginx, etc.) | https://www.dockerstatus.com |

---

## 7. Decision Log

Key architectural decisions and their rationale:

| Decision | Rationale | Alternatives Considered |
|----------|-----------|------------------------|
| **Rust for indexer** | Memory safety, performance, WASM/Soroban ecosystem alignment | Go, Node.js |
| **SvelteKit for frontend** | Small bundle size, server-side rendering, TypeScript native | Next.js, Nuxt |
| **PostgreSQL for off-chain store** | ACID compliance, powerful indexing, `pg_stat_statements` for perf analysis | MongoDB, SQLite |
| **Redis for caching** | Sub-millisecond reads, TTL support, LRU eviction; graceful fallback if unavailable | In-memory (non-persistent), Memcached |
| **Docker Compose (not K8s)** | Simpler ops for single-host deployment; `k8s/` dir exists for future scale | Kubernetes (premature at current scale) |
| **Let's Encrypt for TLS** | Free, automated, widely trusted | Commercial CA, self-signed (unacceptable) |
| **Immutable contract design** | User trust — no admin can change contract logic post-audit | Upgradeable proxy pattern |

---

*© 2026 StellarEscrow*
