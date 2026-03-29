# ⚙️ Operational Procedures

> **StellarEscrow** — Day-to-Day Platform Management & Maintenance  
> Version: 1.0 | Audience: On-Call Engineers, Platform Team, DevOps  
> Review Cadence: Quarterly or after major incidents

---

## Table of Contents

- [Daily Operations](#1-daily-operations)
- [Backup Operations](#2-backup-operations)
- [Security Operations](#3-security-operations)
- [Performance Operations](#4-performance-operations)
- [Monitoring & Alerting](#5-monitoring--alerting)
- [Change Management](#6-change-management)

---

## 1. Daily Operations

### 1.1 Morning Health Check (Daily)

Run this at the start of every on-call shift:

```bash
# 1. Verify all containers are running and healthy
docker compose ps

# 2. Check for overnight alerts
#    → Alertmanager: http://localhost:9093
#    → Grafana:      http://localhost:3002

# 3. Verify backup ran successfully
tail -20 /var/log/stellar-backup.log

# 4. Check disk usage
df -h
du -sh /var/lib/docker/volumes/

# 5. Check latest indexer lag
curl -s http://localhost:3000/health | python3 -m json.tool
```

### 1.2 Scheduled Maintenance Cron Reference

| Schedule | Task | Script | Log File |
|----------|------|--------|----------|
| Daily @ 02:00 UTC | Database backup to S3 | `scripts/backup.sh` | `/var/log/stellar-backup.log` |
| Monthly @ 04:00 UTC (1st) | Disaster recovery test | `scripts/dr-test.sh` | `/var/log/stellar-dr-test.log` |
| Twice daily (00:00, 12:00) | TLS certificate renewal check | `scripts/renew-certs.sh` | `/var/log/cert-renewal.log` |
| Weekly (Sunday 03:00) | Performance bottleneck analysis | `scripts/perf-analyze.sh` | `/var/log/perf-report.log` |
| Quarterly | API key rotation | Manual (see [Section 3.2](#32-api-key-rotation-quarterly)) | Change management log |

```bash
# Full crontab reference
0 2 * * *    /opt/stellarescrow/scripts/backup.sh >> /var/log/stellar-backup.log 2>&1
0 4 1 * *    /opt/stellarescrow/scripts/dr-test.sh >> /var/log/stellar-dr-test.log 2>&1
0 0,12 * * * /opt/stellarescrow/scripts/renew-certs.sh >> /var/log/cert-renewal.log 2>&1
0 3 * * 0    /opt/stellarescrow/scripts/perf-analyze.sh >> /var/log/perf-report.log 2>&1
```

---

## 2. Backup Operations

### 2.1 Automated Daily Backup

The `backup` Docker service runs `scripts/backup.sh` daily at **02:00 UTC**. It performs `pg_dump`, compresses with gzip, uploads to S3, and sends webhook notifications on success or failure.

#### Required Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `S3_BUCKET` | Yes (prod) | S3 URI (e.g., `s3://stellarescrow-backups`) |
| `RETENTION_DAYS` | No | Days to keep local backups (default: 30) |
| `BACKUP_ALERT_WEBHOOK` | No | Slack/webhook URL for backup status notifications |

#### Manual Backup Trigger

```bash
# Trigger a backup immediately (via Docker)
docker compose run --rm backup bash /scripts/backup.sh

# Or directly on host
DATABASE_URL=<url> S3_BUCKET=s3://stellarescrow-backups ./scripts/backup.sh
```

### 2.2 Backup Verification

```bash
# List recent S3 backups
aws s3 ls s3://stellarescrow-backups/ | sort | tail -5

# Verify backup file is non-zero and can be decompressed
aws s3 cp s3://stellarescrow-backups/stellar_escrow_<TIMESTAMP>.sql.gz /tmp/test.sql.gz
gzip -t /tmp/test.sql.gz && echo "Backup file is valid"
```

The backup script also reports a `backup_success` metric to `/api/metrics` — monitor via Grafana.

### 2.3 Monthly DR Test

Run the disaster recovery test on the 1st of each month:

```bash
# Test using the latest backup
./scripts/dr-test.sh

# Test using a specific backup
./scripts/dr-test.sh /var/backups/stellarescrow/stellar_escrow_<TIMESTAMP>.sql.gz

# Test using S3 backup
S3_BUCKET=s3://stellarescrow-backups ./scripts/dr-test.sh
```

The DR test script:
1. Spins up an isolated Postgres container
2. Restores the backup into it
3. Verifies row counts in `trades` and `events` tables
4. Checks schema integrity (≥ 3 tables)
5. Reports `pass`/`fail` to `/api/metrics` and fires an alert webhook

---

## 3. Security Operations

### 3.1 TLS Certificate Management

Certificates are automatically renewed by the `certbot` container every 12 hours. Verify certificate status regularly:

```bash
# Check certificate expiry
echo | openssl s_client -connect stellarescrow.app:443 2>/dev/null \
  | openssl x509 -noout -dates

# Force renewal if < 30 days remaining
docker compose run --rm certbot renew --force-renewal
docker compose exec nginx nginx -s reload
```

### 3.2 API Key Rotation (Quarterly)

> ℹ️ Rotate API keys quarterly and **immediately** if a key is suspected compromised. Coordinate with all API consumers before rotating to avoid service interruptions.

**Step 1 — Generate new API keys**

```bash
# Generate cryptographically secure 32-byte keys
openssl rand -hex 32   # Run once per key needed
```

**Step 2 — Notify API consumers**

Notify all API users of the upcoming key rotation with **1 week notice**.

**Step 3 — Add new keys alongside old ones (transition window)**

```bash
# Edit .env — add new keys alongside old ones
API_KEYS=old-key,new-key-1,new-key-2
```

**Step 4 — Restart API to pick up new keys**

```bash
docker compose restart api
```

**Step 5 — Remove old keys after all consumers have switched**

```bash
API_KEYS=new-key-1,new-key-2
docker compose restart api
```

### 3.3 Secrets Rotation Schedule

| Secret | Rotation Frequency | Procedure |
|--------|-------------------|-----------|
| `POSTGRES_PASSWORD` | Annually or on breach | Update `.env`, restart postgres, update `DATABASE_URL`, restart all services |
| `REDIS_PASSWORD` | Annually or on breach | Update `.env`, restart redis; all services reconnect automatically |
| `API_KEYS` | Quarterly | See [Section 3.2](#32-api-key-rotation-quarterly) |
| `ADMIN_KEYS` | Quarterly | Same as API key rotation |
| `GRAFANA_ADMIN_PASSWORD` | Quarterly | Update via Grafana UI (Profile → Change Password), then update `.env` |
| S3 credentials | Per IAM policy | Update AWS credentials/IAM role attached to instance |

---

## 4. Performance Operations

### 4.1 Weekly Performance Review

```bash
# Run the bottleneck analysis script
./scripts/perf-analyze.sh

# JSON output for dashboards
./scripts/perf-analyze.sh --json
```

The script checks:
1. **Slow queries** — `pg_stat_statements` top 10 by mean execution time
2. **Index usage** — tables with low index-scan ratio (candidates for new indexes)
3. **API latency** — live data from `/health/metrics`
4. **Container resources** — CPU/memory via `docker stats`
5. **Redis hit rate** — `keyspace_hits` vs `keyspace_misses`

#### Performance Thresholds

| Metric | Warning | Critical — Action Required |
|--------|---------|---------------------------|
| TTFB | > 400ms | > 800ms — scale or optimize |
| DB query mean (hot paths) | > 20ms | > 50ms — add index or optimize query |
| Redis hit rate | < 70% | < 60% — review TTL config or memory |
| Indexer memory | > 200MB | > 400MB — investigate leak, restart if needed |
| Disk usage | > 70% | > 85% — expand disk or prune volumes |
| PostgreSQL connections | > 8 (of max 10) | > 9 — increase `max_connections` or scale |

### 4.2 Database Maintenance

```bash
# Manual VACUUM ANALYZE (run during low-traffic periods)
docker compose exec postgres psql -U indexer -d stellar_escrow \
  -c "VACUUM ANALYZE;"

# Check table bloat
docker compose exec postgres psql -U indexer -d stellar_escrow -c \
  "SELECT schemaname, tablename, n_dead_tup, n_live_tup
   FROM pg_stat_user_tables
   ORDER BY n_dead_tup DESC;"

# Enable pg_stat_statements (one-time; required for perf-analyze.sh)
docker compose exec postgres psql -U indexer -d stellar_escrow \
  -c "CREATE EXTENSION IF NOT EXISTS pg_stat_statements;"
```

---

## 5. Monitoring & Alerting

### 5.1 Grafana Dashboards

Access Grafana at `http://localhost:3002` (internal) or `https://grafana.stellarescrow.app` (if configured).  
Default login: uses `GRAFANA_ADMIN_PASSWORD` from `.env`.

| Dashboard | Description | Key Panels |
|-----------|-------------|-----------|
| Infrastructure Overview | All-up system health | Service status, request rate, error rate, resource usage |
| Indexer Performance | Blockchain event processing | Ledger lag, events/sec, DB write latency |
| API Latency | API gateway response times | P50/P95/P99 by endpoint, error breakdown |
| Database Metrics | PostgreSQL performance | Query latency, connection count, cache hit rate, bloat |
| Trade Analytics | Business metrics | Volume, success rate, dispute rate, fees collected |
| Web Vitals | Frontend performance | TTFB, LCP, FID, CLS by page |

### 5.2 Alert Response SLAs

| Alert Name | Severity | Response SLA | Runbook |
|------------|----------|-------------|---------|
| `ServiceDown` | P1 | 15 minutes | [RB-001](03-TROUBLESHOOTING.md#rb-001-postgresql-failure) – [RB-005](03-TROUBLESHOOTING.md#rb-005-redis-cache-failure) |
| `IndexerLag > 5 minutes` | P2 | 1 hour | [RB-002](03-TROUBLESHOOTING.md#rb-002-indexer-not-processing-events) |
| `TLSCertExpiringSoon (< 14 days)` | P2 | 4 hours | [RB-004](03-TROUBLESHOOTING.md#rb-004-nginx--tls-issues) |
| `DBQueryLatency > 50ms` | P3 | 4 hours | [RB-006](03-TROUBLESHOOTING.md#rb-006-high-database-query-latency) |
| `RedisHitRate < 60%` | P3 | 4 hours | [RB-005](03-TROUBLESHOOTING.md#rb-005-redis-cache-failure) |
| `BackupFailed` | P2 | 1 hour | [Section 2.1](#21-automated-daily-backup) |
| `DiskUsage > 85%` | P2 | 2 hours | [Section 4.1](#41-weekly-performance-review) |

---

## 6. Change Management

### 6.1 Deployment Approval Process

| Change Type | Approval Required | Process |
|-------------|------------------|---------|
| Config-only change (env vars) | On-call lead | Update `.env`, rolling restart, verify health checks |
| Application update (Docker image) | On-call lead + peer review | PR review, staging validation, blue/green deploy |
| Database migration | Platform lead | Migration script review, staging test, rollback plan documented |
| New smart contract deployment | Platform lead + CTO | Audit required, testnet validation, mainnet deploy with multisig |
| Infrastructure change (`docker-compose`) | Platform lead | Review, staging deploy, production deploy with monitoring |

### 6.2 Post-Incident Review

After every **P1 or P2** incident, conduct a blameless post-incident review within 48 hours. Document:

- **Timeline** of events (UTC timestamps)
- **Root cause** analysis
- **Impact assessment** (duration, affected users, financial impact if any)
- **Immediate remediation** steps taken
- **Long-term corrective actions** with owners and due dates
- **Detection gap** — could monitoring have caught this earlier?
- **Runbook improvements** required

---

*© 2026 StellarEscrow*
