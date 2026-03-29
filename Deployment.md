# 🚀 Deployment Guide

> **StellarEscrow** — Step-by-Step Production & Environment Setup  
> Version: 1.0 | Audience: DevOps Engineers, Platform Team

---

## Table of Contents

- [Prerequisites & System Requirements](#1-prerequisites--system-requirements)
- [Environment Configuration](#2-environment-configuration)
- [Smart Contract Deployment](#3-smart-contract-deployment)
- [Docker Stack Deployment](#4-docker-stack-deployment)
- [Database Migration](#5-database-migration)
- [Upgrading](#6-upgrading)
- [Rollback Procedures](#7-rollback-procedures)
- [Post-Deployment Verification Checklist](#8-post-deployment-verification-checklist)

---

## 1. Prerequisites & System Requirements

### 1.1 Minimum Server Specification

| Resource | Minimum | Recommended (Production) |
|----------|---------|--------------------------|
| CPU | 2 vCPU | 4 vCPU |
| RAM | 4 GB | 8 GB |
| Disk | 20 GB SSD | 100 GB SSD |
| OS | Ubuntu 22.04 LTS | Ubuntu 22.04 LTS |
| Network | 100 Mbps | 1 Gbps |
| Open Ports | 80, 443 (inbound) | 80, 443 (inbound) |

### 1.2 Required Software

- Docker Engine 24.x or later
- Docker Compose v2.x (`docker compose` plugin)
- Soroban CLI (for contract deployment only)
- AWS CLI v2 (for S3 backup integration)
- Git 2.x

```bash
# Install Docker (Ubuntu)
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# Install Soroban CLI
cargo install --locked soroban-cli
```

---

## 2. Environment Configuration

### 2.1 Environment File Setup

StellarEscrow uses environment-specific `.env` files. **Never commit secrets to source control** — `.gitignore` excludes all `.env.*` files except `.env.example`.

```bash
# Copy the example and populate secrets
cp .env.example .env
nano .env   # or your preferred editor
```

### 2.2 Required Environment Variables

| Variable | Required | Example Value | Notes |
|----------|----------|---------------|-------|
| `POSTGRES_PASSWORD` | Yes | `use-strong-random-secret` | Minimum 24 characters; alphanumeric only |
| `REDIS_PASSWORD` | Yes | `another-random-secret` | Required by Redis `requirepass` config |
| `DATABASE_URL` | Yes | `postgres://indexer:PGPW@postgres:5432/stellar_escrow` | Uses `POSTGRES_PASSWORD` value |
| `API_KEYS` | Yes | `key1,key2,key3` | Comma-separated; rotate quarterly |
| `ADMIN_KEYS` | Yes | `admin-key-here` | Separate from `API_KEYS`; strictly limited |
| `GRAFANA_ADMIN_PASSWORD` | Yes | `grafana-admin-pw` | Initial Grafana login password |
| `S3_BUCKET` | Yes (prod) | `s3://stellarescrow-backups` | S3 URI for database backups |
| `BACKUP_ALERT_WEBHOOK` | No | `https://hooks.slack.com/...` | Slack webhook for backup alerts |
| `CDN_PURGE_URL` | No | `https://api.cdn.com/purge` | CDN purge endpoint after indexing |
| `CDN_PURGE_TOKEN` | No | `cdn-token-here` | Bearer token for CDN purge API |
| `GRAFANA_URL` | No | `https://grafana.stellarescrow.app` | Public URL for Grafana links |
| `RETENTION_DAYS` | No | `30` | Backup retention in days (default: 30) |

---

## 3. Smart Contract Deployment

> ⚠️ **IMPORTANT:** Contract deployment is **one-time per environment**. After deployment, the contract address is immutable. Record the contract ID immediately and store it in your secrets manager.

### 3.1 Build the WASM Contract

```bash
cd contract
cargo build --target wasm32-unknown-unknown --release
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/stellar_escrow.wasm
```

### 3.2 Deploy to Testnet

```bash
# Fund a testnet account (Friendbot)
curl "https://friendbot.stellar.org?addr=$(soroban config identity address dev)"

# Deploy to testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_escrow.optimized.wasm \
  --source dev \
  --network testnet

# Output: CONTRACT_ID=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
# Save this — you need it in .env.staging
```

### 3.3 Deploy to Mainnet

```bash
# Requires funded mainnet account
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_escrow.optimized.wasm \
  --source prod-deployer \
  --network mainnet

# Update .env.production with the mainnet CONTRACT_ID
```

---

## 4. Docker Stack Deployment

### 4.1 Initial Production Deployment

**Step 1 — Clone the repository**

```bash
git clone https://github.com/Mystery-CLI/StellarEscrow.git
cd StellarEscrow
```

**Step 2 — Create and populate the `.env` file**

```bash
cp .env.example .env
# Edit .env with all required variables (see Section 2.2)
```

**Step 3 — Start infrastructure services first (database + cache)**

```bash
docker compose up -d postgres redis
# Wait for health checks to pass (30–60 seconds)
docker compose ps   # All should show "healthy"
```

**Step 4 — Start the indexer and API**

```bash
docker compose up -d indexer api
docker compose logs -f indexer   # Watch for "Listening on 0.0.0.0:3000"
```

**Step 5 — Start the frontend and Nginx**

```bash
docker compose up -d client nginx
```

**Step 6 — Start the monitoring stack**

```bash
docker compose up -d prometheus grafana loki promtail alertmanager
```

**Step 7 — Start backup and certificate services**

```bash
docker compose up -d backup certbot
```

**Step 8 — Verify all services are healthy**

```bash
docker compose ps
curl -f http://localhost/health    # Should return 200 OK
```

### 4.2 TLS Certificate Setup (First-Time Only)

> ⚠️ Complete this **BEFORE** routing public traffic to the server. Nginx will fail to start if certificate files are missing.

```bash
# Issue initial certificate
docker compose run --rm certbot certonly \
  --webroot -w /var/www/certbot \
  -d stellarescrow.app \
  -d www.stellarescrow.app \
  --email ops@stellarescrow.app \
  --agree-tos --non-interactive

# Reload Nginx to pick up the new certificate
docker compose exec nginx nginx -s reload
```

### 4.3 Staging Deployment

Staging uses the same `docker-compose.yml` with `.env.staging` overrides. The Stellar testnet is used instead of mainnet.

```bash
cp .env.staging .env
docker compose up -d
```

---

## 5. Database Migration

### 5.1 Running Migrations

Migrations run automatically when the indexer starts for the first time. On subsequent starts, the indexer checks and applies pending migrations.

```bash
# Verify migration status
docker compose exec indexer sh -c "RUST_LOG=debug cargo run -- migrate --check"

# Run migrations manually if needed
docker compose exec indexer sh -c "cargo run -- migrate"
```

---

## 6. Upgrading

### 6.1 Rolling Service Update

```bash
# Pull latest code
git pull origin main

# Rebuild changed services
docker compose build indexer api client

# Rolling restart — one service at a time
docker compose up -d --no-deps indexer
docker compose ps indexer          # Verify health before proceeding
docker compose up -d --no-deps api
docker compose up -d --no-deps client
```

> **Note:** The smart contract cannot be upgraded — deploy a new contract and update `CONTRACT_ID` if breaking changes are required.

---

## 7. Rollback Procedures

### 7.1 Application Rollback

```bash
# Identify the previous image tag
docker images | grep stellar-escrow

# Roll back indexer to previous build
docker compose stop indexer
docker tag stellar-escrow-indexer:previous stellar-escrow-indexer:latest
docker compose up -d indexer
```

### 7.2 Database Rollback

If a migration must be rolled back, restore from the most recent backup taken before the migration:

```bash
# Stop services to prevent writes during restore
docker compose stop indexer api

# Restore backup (see Troubleshooting Runbook — RB-008 for full procedure)
./scripts/restore.sh /var/backups/stellarescrow/stellar_escrow_<TIMESTAMP>.sql.gz

# Restart services
docker compose start indexer api
```

---

## 8. Post-Deployment Verification Checklist

| Check | Command | Expected Result |
|-------|---------|----------------|
| Nginx health | `curl -sf https://stellarescrow.app/health` | HTTP 200 |
| Indexer health | `curl -sf http://localhost:3000/health` | HTTP 200 + JSON status |
| API health | `curl -sf http://localhost:4000/health` | HTTP 200 + JSON status |
| PostgreSQL | `docker compose exec postgres pg_isready -U indexer` | `"accepting connections"` |
| Redis | `docker compose exec redis redis-cli ping` | `PONG` |
| TLS certificate | `curl -I https://stellarescrow.app` | HTTP 200, valid cert |
| Grafana | `curl -sf http://localhost:3002` | HTTP 200 |
| Prometheus | `curl -sf http://localhost:9090/-/healthy` | HTTP 200 |
| Backup service | `docker compose ps backup` | `"running"` status |
| Smoke test | `soroban contract invoke -- get_funding_preview` | Valid FundingPreview JSON |

---

*© 2026 StellarEscrow*
