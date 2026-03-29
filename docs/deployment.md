# Deployment Guide

> StellarEscrow — Environment Setup, CI/CD, and Production Deployment  
> Audience: DevOps Engineers, Platform Team

## Table of Contents

- [Prerequisites](#prerequisites)
- [Environment Configuration](#environment-configuration)
- [Smart Contract Deployment](#smart-contract-deployment)
- [Docker Stack Deployment](#docker-stack-deployment)
- [Database Migrations](#database-migrations)
- [CI/CD Pipeline](#cicd-pipeline)
- [Upgrading Services](#upgrading-services)
- [Rollback Procedures](#rollback-procedures)
- [Post-Deployment Verification](#post-deployment-verification)

---

## Prerequisites

### Minimum Server Specification

| Resource | Minimum | Recommended (Production) |
|----------|---------|--------------------------|
| CPU | 2 vCPU | 4 vCPU |
| RAM | 4 GB | 8 GB |
| Disk | 20 GB SSD | 100 GB SSD |
| OS | Ubuntu 22.04 LTS | Ubuntu 22.04 LTS |
| Open Ports | 80, 443 (inbound) | 80, 443 (inbound) |

### Required Software

```bash
# Docker Engine 24.x
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# Soroban CLI (contract deployment only)
cargo install --locked soroban-cli

# AWS CLI v2 (S3 backup integration)
# https://docs.aws.amazon.com/cli/latest/userguide/install-cliv2.html
```

---

## Environment Configuration

StellarEscrow uses environment-specific `.env` files. **Never commit secrets to source control** — `.gitignore` excludes all `.env.*` files except `.env.example`.

```bash
cp .env.example .env
# Populate all required variables
```

### Required Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `POSTGRES_PASSWORD` | Yes | Min 24 chars; alphanumeric only |
| `REDIS_PASSWORD` | Yes | Required by Redis `requirepass` |
| `DATABASE_URL` | Yes | `postgres://indexer:<PGPW>@postgres:5432/stellar_escrow` |
| `CONTRACT_ID` | Yes | Soroban contract address (from deployment step) |
| `STELLAR_NETWORK` | Yes | `TESTNET` or `MAINNET` |
| `HORIZON_URL` | Yes | Stellar Horizon RPC endpoint |
| `API_KEYS` | Yes | Comma-separated; rotate quarterly |
| `ADMIN_KEYS` | Yes | Separate from `API_KEYS`; strictly limited |
| `GRAFANA_ADMIN_PASSWORD` | Yes | Initial Grafana login password |
| `S3_BUCKET` | Yes (prod) | `s3://stellarescrow-backups` |
| `BACKUP_ALERT_WEBHOOK` | No | Slack webhook for backup alerts |
| `REDIS_URL` | No | Enables API response caching if set |
| `RETENTION_DAYS` | No | Backup retention in days (default: 30) |

Environment-specific config files are in `config/environments/` (`development.toml`, `staging.toml`, `production.toml`).

---

## Smart Contract Deployment

> ⚠️ Contract deployment is **one-time per environment**. The contract address is immutable after deployment — record the `CONTRACT_ID` immediately and store it in your secrets manager.

### Build

```bash
cd contract
cargo build --target wasm32-unknown-unknown --release
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/stellar_escrow.wasm
```

### Deploy to Testnet (Staging)

```bash
# Fund a testnet account via Friendbot
curl "https://friendbot.stellar.org?addr=$(soroban config identity address dev)"

soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_escrow.optimized.wasm \
  --source dev \
  --network testnet
# → Save the output CONTRACT_ID to .env.staging
```

### Deploy to Mainnet (Production)

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_escrow.optimized.wasm \
  --source prod-deployer \
  --network mainnet
# → Save the output CONTRACT_ID to .env.production
```

---

## Docker Stack Deployment

### Initial Production Deployment

```bash
# 1. Clone the repository
git clone https://github.com/Mystery-CLI/StellarEscrow.git
cd StellarEscrow

# 2. Populate environment
cp .env.example .env
# Edit .env with all required variables

# 3. Start infrastructure first (database + cache)
docker compose up -d postgres redis
docker compose ps   # Wait until both show "healthy" (30–60s)

# 4. Start the indexer and API
docker compose up -d indexer api
docker compose logs -f indexer   # Watch for "Listening on 0.0.0.0:3000"

# 5. Start the frontend and Nginx
docker compose up -d client nginx

# 6. Start the monitoring stack
docker compose up -d prometheus grafana loki promtail alertmanager

# 7. Start backup and certificate services
docker compose up -d backup certbot

# 8. Verify
docker compose ps
curl -f http://localhost/health
```

### TLS Certificate Setup (First-Time Only)

> Complete this **before** routing public traffic. Nginx will fail to start if certificate files are missing.

```bash
docker compose run --rm certbot certonly \
  --webroot -w /var/www/certbot \
  -d stellarescrow.app \
  -d www.stellarescrow.app \
  --email ops@stellarescrow.app \
  --agree-tos --non-interactive

docker compose exec nginx nginx -s reload
```

### Staging Deployment

```bash
cp .env.staging .env
docker compose up -d
```

---

## Database Migrations

Migrations in `indexer/migrations/` run automatically on indexer startup. To check or run manually:

```bash
# Check pending migrations
docker compose exec indexer sh -c "RUST_LOG=debug cargo run -- migrate --check"

# Run manually
docker compose exec indexer sh -c "cargo run -- migrate"
```

---

## CI/CD Pipeline

The GitHub Actions workflows in `.github/workflows/` automate the full delivery pipeline:

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `rust.yml` | Push to `main`, PRs | Cargo build + test for contract and indexer |
| `code-quality.yml` | Push, PRs | ESLint, Prettier, Clippy |
| `security-scan.yml` | Push to `main`, schedule | Dependency audit, secret scanning |
| `e2e.yml` | Push to `main` | Playwright end-to-end tests |
| `terraform.yml` | Push to `main` | Terraform plan/apply for infra changes |
| `promote.yml` | Manual dispatch | Promote staging image to production |
| `regression.yml` | Schedule | Regression test suite |
| `uat.yml` | Manual dispatch | User acceptance test scenarios |

### Promotion Flow

```
feature branch → PR → CI checks pass → merge to main
    → staging deploy (automatic)
    → manual approval
    → production deploy (promote.yml)
```

### Required GitHub Secrets

Configure these in your repository's Settings → Secrets:

```
POSTGRES_PASSWORD
REDIS_PASSWORD
DATABASE_URL
CONTRACT_ID
API_KEYS
ADMIN_KEYS
S3_BUCKET
AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY
GRAFANA_ADMIN_PASSWORD
```

---

## Upgrading Services

```bash
# Pull latest code
git pull origin main

# Rebuild changed services
docker compose build indexer api client

# Rolling restart — one service at a time to avoid downtime
docker compose up -d --no-deps indexer
docker compose ps indexer          # Verify healthy before proceeding
docker compose up -d --no-deps api
docker compose up -d --no-deps client
```

> **Note:** The smart contract cannot be upgraded in-place. Deploy a new contract and update `CONTRACT_ID` in `.env` if breaking changes are required.

---

## Rollback Procedures

### Application Rollback

```bash
# Identify the previous image tag
docker images | grep stellar-escrow

# Roll back a service to the previous build
docker compose stop indexer
docker tag stellar-escrow-indexer:previous stellar-escrow-indexer:latest
docker compose up -d indexer
```

### Database Rollback

```bash
# Stop write services to prevent conflicts
docker compose stop indexer api

# Restore from backup (see runbooks/troubleshooting.md — RB-008)
./scripts/restore.sh /var/backups/stellarescrow/stellar_escrow_<TIMESTAMP>.sql.gz

# Restart services
docker compose start indexer api
```

---

## Post-Deployment Verification

| Check | Command | Expected |
|-------|---------|----------|
| Nginx health | `curl -sf https://stellarescrow.app/health` | HTTP 200 |
| Indexer health | `curl -sf http://localhost:3000/health` | HTTP 200 + JSON |
| API health | `curl -sf http://localhost:4000/health` | HTTP 200 + JSON |
| PostgreSQL | `docker compose exec postgres pg_isready -U indexer` | `accepting connections` |
| Redis | `docker compose exec redis redis-cli ping` | `PONG` |
| TLS certificate | `curl -I https://stellarescrow.app` | HTTP 200, valid cert |
| Grafana | `curl -sf http://localhost:3002` | HTTP 200 |
| Prometheus | `curl -sf http://localhost:9090/-/healthy` | HTTP 200 |
| Backup service | `docker compose ps backup` | `running` |
| Contract smoke test | `soroban contract invoke -- get_funding_preview` | Valid JSON |

---

*See also: [Architecture](architecture.md) | [Operations](operations.md) | [Troubleshooting](runbooks/troubleshooting.md)*
