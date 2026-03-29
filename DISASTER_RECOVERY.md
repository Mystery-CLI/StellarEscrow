# StellarEscrow — Disaster Recovery Plan

**Version:** 1.0 | **Priority:** High | **Owner:** Platform Team

---

## 1. Objectives

| Metric | Target |
|--------|--------|
| Recovery Time Objective (RTO) | ≤ 2 hours |
| Recovery Point Objective (RPO) | ≤ 24 hours (daily backup) |
| Backup Retention | 30 days |

---

## 2. System Components

| Component | Technology | Backup Method |
|-----------|-----------|---------------|
| Database | PostgreSQL 15 | `pg_dump` → gzip → S3 |
| Smart Contract | Stellar blockchain | Immutable on-chain — no backup needed |
| Static Frontend | nginx / CDN | Rebuilt from source |
| Indexer | Rust binary | Rebuilt from source |
| TLS Certificates | Let's Encrypt | Auto-renewed; backed up in `letsencrypt` volume |

---

## 3. Backup System

### Automated Daily Backup
The `backup` Docker service runs `scripts/backup.sh` daily at 02:00 UTC.

```bash
# Manual backup trigger
docker compose run --rm backup bash /scripts/backup.sh

# Or directly on the host
DATABASE_URL=<url> S3_BUCKET=s3://stellarescrow-backups ./scripts/backup.sh
```

### Required Environment Variables
```
DATABASE_URL          PostgreSQL connection string
S3_BUCKET             S3 bucket URI (e.g. s3://stellarescrow-backups)
RETENTION_DAYS        Local backup retention (default: 30)
BACKUP_ALERT_WEBHOOK  Slack/webhook URL for alerts
```

### Backup Verification
Each backup run reports a `backup_success` metric to `/api/metrics`. Monitor via your observability stack.

---

## 4. Recovery Procedures

### 4.1 Database Failure

**Symptoms:** Indexer cannot connect to Postgres; API returns 5xx errors.

```bash
# 1. Identify latest backup
ls -lt /var/backups/stellarescrow/ | head -5
# or from S3:
aws s3 ls s3://stellarescrow-backups/ | sort | tail -5

# 2. Restore
./scripts/restore.sh /var/backups/stellarescrow/stellar_escrow_<TIMESTAMP>.sql.gz
# or from S3:
./scripts/restore.sh s3://stellarescrow-backups/stellar_escrow_<TIMESTAMP>.sql.gz

# 3. Restart indexer
docker compose restart indexer
```

### 4.2 Indexer / API Failure

**Symptoms:** Frontend shows offline; `/api/health` returns non-200.

```bash
# Restart the service
docker compose restart indexer

# If image is corrupt, rebuild
docker compose build indexer && docker compose up -d indexer
```

### 4.3 Full Host Failure

```bash
# 1. Provision new host and install Docker
# 2. Clone repository
git clone https://github.com/Mystery-CLI/StellarEscrow.git && cd StellarEscrow

# 3. Set environment variables (DATABASE_URL, S3_BUCKET, CDN_PURGE_URL, etc.)
cp .env.example .env && nano .env

# 4. Start services
docker compose up -d

# 5. Restore latest database backup
./scripts/restore.sh s3://stellarescrow-backups/<latest>.sql.gz

# 6. Reissue TLS certificate if needed
docker compose run --rm certbot certonly --webroot -w /var/www/certbot -d stellarescrow.app
```

### 4.4 Stellar Network / Contract Issue

The escrow contract is immutable on the Stellar blockchain. If the network is degraded:
- The indexer will pause ingestion and retry automatically (`poll_interval_seconds = 5`)
- Existing trade state is preserved in PostgreSQL
- No recovery action required — resume is automatic when the network recovers

---

## 5. Recovery Testing

Run the DR test monthly and after every major deployment:

```bash
# Test against latest local backup
./scripts/dr-test.sh

# Test against a specific backup
./scripts/dr-test.sh /var/backups/stellarescrow/stellar_escrow_<TIMESTAMP>.sql.gz

# Test against S3 backup
S3_BUCKET=s3://stellarescrow-backups ./scripts/dr-test.sh
```

The test script:
1. Spins up an isolated Postgres container
2. Restores the backup into it
3. Verifies row counts in `trades` and `events` tables
4. Checks schema integrity (≥3 tables)
5. Reports `pass`/`fail` to `/api/metrics` and fires a webhook alert

**Recommended cron schedule:**
```
0 4 1 * * /opt/stellarescrow/scripts/dr-test.sh >> /var/log/stellar-dr-test.log 2>&1
```

---

## 6. Runbook Contacts

| Role | Responsibility |
|------|---------------|
| On-call Engineer | First responder — execute recovery procedures |
| Platform Lead | Escalation for full host failure or data loss |
| Stellar Network | https://status.stellar.org for network incidents |

---

## 7. Backup Cron Reference

```
# Daily database backup at 02:00 UTC
0 2 * * * /opt/stellarescrow/scripts/backup.sh >> /var/log/stellar-backup.log 2>&1

# Monthly DR test at 04:00 UTC on the 1st
0 4 1 * * /opt/stellarescrow/scripts/dr-test.sh >> /var/log/stellar-dr-test.log 2>&1

# Certificate renewal check twice daily
0 0,12 * * * /opt/stellarescrow/scripts/renew-certs.sh >> /var/log/cert-renewal.log 2>&1
```
