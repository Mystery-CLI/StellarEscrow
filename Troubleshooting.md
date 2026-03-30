# 🔧 Troubleshooting Runbooks

> **StellarEscrow** — Incident Response & Operational Procedures  
> Version: 1.0 | Audience: On-Call Engineers, SRE Team  
> **RTO Target:** ≤ 2 hours | **RPO Target:** ≤ 24 hours

---

## Quick Reference

| Resource | URL |
|----------|-----|
| Stellar Network Status | https://status.stellar.org |
| Grafana | http://localhost:3002 |
| Alertmanager | http://localhost:9093 |
| Prometheus | http://localhost:9090 |

---

## Table of Contents

- [Incident Classification](#1-incident-classification)
- [First-Responder Checklist](#2-first-responder-checklist)
- [Service-Specific Runbooks](#3-service-specific-runbooks)
  - [RB-001: PostgreSQL Failure](#rb-001-postgresql-failure)
  - [RB-002: Indexer Not Processing Events](#rb-002-indexer-not-processing-events)
  - [RB-003: API Gateway 5xx Errors](#rb-003-api-gateway-returning-5xx-errors)
  - [RB-004: Nginx / TLS Issues](#rb-004-nginx--tls-issues)
  - [RB-005: Redis Cache Failure](#rb-005-redis-cache-failure)
  - [RB-006: High Database Query Latency](#rb-006-high-database-query-latency)
- [Stellar Network Incidents](#4-stellar-network-incidents)
  - [RB-007: Stellar Network Degraded](#rb-007-stellar-network-degraded)
- [Database Backup & Restore](#5-database-backup--restore)
  - [RB-008: Database Restore](#rb-008-database-restore-from-backup)
- [Full Host Failure Recovery](#6-full-host-failure-recovery)
  - [RB-009: Complete Host Rebuild](#rb-009-complete-host-rebuild)
- [Escalation Matrix](#7-escalation-matrix)

---

## 1. Incident Classification

| Severity | Description | Response Time | Examples |
|----------|-------------|---------------|---------|
| **P1 — Critical** | Complete service outage; active fund loss risk | 15 minutes | Nginx down, DB unrecoverable, contract exploit |
| **P2 — High** | Major feature broken; many users affected | 1 hour | Indexer stopped, API all-5xx, TLS expired |
| **P3 — Medium** | Degraded performance or partial outage | 4 hours | High latency, Redis down, cache miss spike |
| **P4 — Low** | Minor issue; workaround available | Next business day | Grafana login issue, log gap, slow CSV export |

---

## 2. First-Responder Checklist

Run this **in order** for any unclassified incident:

```bash
# 1. Check overall service health
docker compose ps

# 2. Check web health
curl -I https://stellarescrow.app/health

# 3. Review recent logs
docker compose logs --since 30m --tail 100

# 4. Check Stellar network
# → https://status.stellar.org

# 5. Check Prometheus alerts
# → http://localhost:9093

# 6. Check disk space
df -h

# 7. Check memory
free -h
```

Then identify the failing component and jump to the relevant runbook below.

---

## 3. Service-Specific Runbooks

---

### RB-001: PostgreSQL Failure

**Symptoms:** Indexer cannot connect to database; API returns HTTP 500 or 503; Grafana shows "DB connection failed" alert.

#### Diagnosis

```bash
# Check container status
docker compose ps postgres

# Check health
docker compose exec postgres pg_isready -U indexer -d stellar_escrow

# Review recent logs
docker compose logs --tail 50 postgres

# Check disk space (full disk is a common cause)
df -h /var/lib/docker
```

#### Resolution

**1. Restart if stopped or unhealthy**

```bash
docker compose restart postgres
# Wait 30 seconds, then verify:
docker compose ps postgres   # Should show "healthy"
```

**2. If restart fails — check for data corruption**

```bash
docker compose logs postgres | grep -i "FATAL\|ERROR\|corrupt"
# If corruption detected, proceed to RB-008 (backup restore)
```

**3. If disk is full — clear Docker build cache**

```bash
docker system prune -f
# If insufficient, expand disk (cloud provider console) then restart
```

**4. After PostgreSQL is healthy — restart dependent services**

```bash
docker compose restart indexer api
```

**5. Verify full recovery**

```bash
curl -sf http://localhost:4000/health
curl -sf http://localhost:3000/health
```

---

### RB-002: Indexer Not Processing Events

**Symptoms:** Trades confirmed on-chain but not appearing in UI; stale trade status; Grafana "Indexer lag" alert fires.

#### Diagnosis

```bash
# Check indexer health
curl http://localhost:3000/health

# Check for errors
docker compose logs --tail 100 indexer | grep -i "error\|WARN\|panic"

# Check Stellar connectivity
curl https://status.stellar.org

# Check last processed ledger
docker compose exec postgres psql -U indexer -d stellar_escrow \
  -c "SELECT MAX(ledger_sequence) FROM events;"
```

#### Resolution

**1. If Stellar network is degraded — wait (no action needed)**

> The indexer retries with exponential backoff. Existing trade state in PostgreSQL is preserved. Do **not** restart the indexer during active network incidents.

**2. If indexer is OOM-killed — increase Docker memory limit**

```bash
# Check last exit code (137 = OOM kill)
docker inspect stellar-escrow-indexer-1 | grep "ExitCode"

# Add to docker-compose.yml under the indexer service:
# deploy:
#   resources:
#     limits:
#       memory: 512M
```

**3. If logs show database connection errors — see [RB-001](#rb-001-postgresql-failure)**

**4. If logs show panics — rebuild and redeploy**

```bash
docker compose build indexer
docker compose up -d --no-deps indexer
```

**5. Trigger event replay if gaps exist**

```bash
curl -X POST http://localhost:3000/events/replay \
  -H "Authorization: Bearer $ADMIN_KEY" \
  -d '{"from_ledger": <START>, "to_ledger": <END>}'
```

---

### RB-003: API Gateway Returning 5xx Errors

**Symptoms:** Frontend displays errors; POST `/api/*` requests fail; API health endpoint returns non-200.

#### Diagnosis

```bash
# Health check
curl -v http://localhost:4000/health

# Check logs for error patterns
docker compose logs --tail 100 api | grep -E "Error|5[0-9]{2}"

# Verify dependencies are reachable
docker compose exec api wget -qO- http://indexer:3000/health
```

#### Resolution

**1. Restart the API**

```bash
docker compose restart api
```

**2. If API cannot reach indexer — restart indexer first**

See [RB-002](#rb-002-indexer-not-processing-events), then restart API.

**3. If API cannot reach database — see [RB-001](#rb-001-postgresql-failure)**

**4. If restart loop — check environment variables**

```bash
docker compose config api   # Verify all required env vars are present
```

---

### RB-004: Nginx / TLS Issues

**Symptoms:** HTTPS site unreachable; SSL certificate error in browser; HTTP 502 Bad Gateway.

#### Diagnosis

```bash
# Test nginx config
docker compose exec nginx nginx -t

# Check certificate validity
echo | openssl s_client -connect stellarescrow.app:443 2>/dev/null \
  | openssl x509 -noout -dates

# Check upstream health (502 usually means backend is down)
curl http://localhost:3000/health   # indexer
curl http://localhost:3001/         # client
```

#### Resolution

**1. HTTP 502 — fix the upstream service first**

Backends are down. Resolve the failing service using RB-001 through RB-003, then Nginx will recover automatically.

**2. Expired TLS certificate — renew manually**

```bash
docker compose run --rm certbot renew --force-renewal
docker compose exec nginx nginx -s reload
```

**3. Nginx config errors — check mounted config files**

```bash
docker compose exec nginx cat /etc/nginx/conf.d/ssl.conf
# Fix errors in ssl.nginx.conf or cdn-headers.nginx.conf on host
docker compose exec nginx nginx -s reload
```

---

### RB-005: Redis Cache Failure

**Symptoms:** Performance degraded; all requests hitting database; Grafana shows "Redis hit rate < 60%" alert; Redis container unhealthy.

> ℹ️ **Safe to ignore briefly:** Redis failure degrades performance but does not cause errors. The system falls back to direct database reads.

#### Resolution

**1. Restart Redis**

```bash
docker compose restart redis
```

**2. Verify Redis is accepting connections**

```bash
docker compose exec redis redis-cli -a "$REDIS_PASSWORD" ping
# Should return: PONG
```

**3. Check Redis memory usage**

```bash
docker compose exec redis redis-cli -a "$REDIS_PASSWORD" info memory \
  | grep used_memory_human
# If at or near maxmemory (256mb), consider increasing in docker-compose.yml
```

---

### RB-006: High Database Query Latency

**Symptoms:** API responses slow (> 800ms); Grafana "DB query mean > 50ms" alert; users report slow page loads.

#### Diagnosis

```bash
# Run the bottleneck analysis script
./scripts/perf-analyze.sh

# Check for long-running queries
docker compose exec postgres psql -U indexer -d stellar_escrow -c \
  "SELECT pid, now()-query_start as duration, query
   FROM pg_stat_activity
   WHERE state='active'
   ORDER BY duration DESC
   LIMIT 10;"

# Check for missing indexes
./scripts/perf-analyze.sh --json | jq '.index_usage'
```

#### Resolution

**1. Kill long-running queries that are blocking**

```sql
SELECT pg_terminate_backend(<pid>);
```

**2. Enable Redis to reduce DB load (if not already enabled)**

```bash
docker compose exec indexer env | grep REDIS
# Ensure REDIS_URL is set
```

**3. VACUUM ANALYZE if table bloat is suspected**

```bash
docker compose exec postgres psql -U indexer -d stellar_escrow \
  -c "VACUUM ANALYZE;"
```

---

## 4. Stellar Network Incidents

---

### RB-007: Stellar Network Degraded

**Symptoms:** Indexer logs show repeated "failed to fetch ledger" errors; no new events being processed; `status.stellar.org` shows active incident.

> ❌ **Do NOT restart the indexer during an active Stellar network incident.** The indexer will resume automatically with no data loss when the network recovers.

#### Actions During Network Degradation

- Monitor https://status.stellar.org for recovery ETA
- Post a status update to users if degradation exceeds 15 minutes
- Existing trade data in PostgreSQL is fully preserved and consistent
- No transactions can be submitted to the contract during outages — this is expected
- All historical trade data remains queryable via the API

#### Post-Recovery Verification

```bash
# Verify indexer has caught up
docker compose logs --tail 50 indexer | grep "ledger"

# Check for any gaps in event sequence
docker compose exec postgres psql -U indexer -d stellar_escrow -c \
  "SELECT MIN(ledger_sequence), MAX(ledger_sequence), COUNT(*)
   FROM events
   WHERE created_at > NOW() - INTERVAL '2 hours';"
```

---

## 5. Database Backup & Restore

---

### RB-008: Database Restore (From Backup)

> ⚠️ **This procedure causes downtime.** Stop indexer and API before restoring to prevent write conflicts.

#### Step 1: Identify the Recovery Point

```bash
# List local backups
ls -lt /var/backups/stellarescrow/ | head -10

# List S3 backups
aws s3 ls s3://stellarescrow-backups/ | sort | tail -10
```

#### Step 2: Stop Write Services

```bash
docker compose stop indexer api
```

#### Step 3: Run Restore Script

```bash
# From local file
./scripts/restore.sh /var/backups/stellarescrow/stellar_escrow_<TIMESTAMP>.sql.gz

# From S3
./scripts/restore.sh s3://stellarescrow-backups/stellar_escrow_<TIMESTAMP>.sql.gz
```

#### Step 4: Verify Data Integrity

```bash
docker compose exec postgres psql -U indexer -d stellar_escrow \
  -c "SELECT COUNT(*) FROM trades; SELECT COUNT(*) FROM events;"
```

#### Step 5: Restart Services

```bash
docker compose start indexer api
docker compose ps   # Wait for health checks to pass
```

#### Step 6: Verify End-to-End

```bash
curl -sf http://localhost:4000/health
curl -sf http://localhost:3000/health
```

---

## 6. Full Host Failure Recovery

---

### RB-009: Complete Host Rebuild

Use this runbook if the production host is unrecoverable. **Target RTO: ≤ 2 hours.**

**Step 1 — Provision a new server** meeting minimum specifications (see [Deployment Guide — Section 1.1](02-DEPLOYMENT.md))

**Step 2 — Install Docker and clone the repository**

```bash
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER && newgrp docker

git clone https://github.com/Mystery-CLI/StellarEscrow.git
cd StellarEscrow
```

**Step 3 — Restore environment secrets from secrets manager**

```bash
cp .env.example .env
# Populate all required variables from your secrets manager
```

**Step 4 — Start all services**

```bash
docker compose up -d
```

**Step 5 — Restore the database from latest S3 backup**

```bash
docker compose stop indexer api
./scripts/restore.sh s3://stellarescrow-backups/<latest>.sql.gz
docker compose start indexer api
```

**Step 6 — Reissue TLS certificate**

```bash
docker compose run --rm certbot certonly \
  --webroot -w /var/www/certbot \
  -d stellarescrow.app \
  --agree-tos --non-interactive \
  --email ops@stellarescrow.app

docker compose exec nginx nginx -s reload
```

**Step 7 — Update DNS if IP address changed**

Update the A/AAAA record for `stellarescrow.app` to the new server IP. DNS propagation may take up to 5 minutes with a low TTL.

**Step 8 — Run post-deployment verification checklist**

See [Deployment Guide — Section 8](02-DEPLOYMENT.md#8-post-deployment-verification-checklist).

---

## 7. Escalation Matrix

| Situation | First Escalation | Second Escalation | External Contact |
|-----------|-----------------|-------------------|-----------------|
| P1 incident > 15 min | On-Call Lead | Platform Lead | — |
| Data loss suspected | Platform Lead (immediate) | CTO | AWS Support (if S3 issue) |
| Stellar network issue | Monitor status.stellar.org | — | https://status.stellar.org |
| Smart contract exploit | Platform Lead (immediate) | Security Team | Stellar Security Disclosure |
| Full host failure | On-Call Engineer | Platform Lead | Cloud provider support |

---

## 8. Infrastructure Alert Runbooks

### RB-010: High CPU / Memory Usage

**Alert:** `HighCPUUsage` / `CriticalCPUUsage` / `HighMemoryUsage` / `CriticalMemoryUsage`

```bash
# Identify top processes
docker stats --no-stream
top -b -n1 | head -20

# Check indexer memory
docker compose exec indexer cat /proc/self/status | grep VmRSS

# Restart the offending service if memory-leaking
docker compose restart indexer
```

### RB-011: Disk Space Warning

**Alert:** `DiskSpaceWarning` / `DiskSpaceCritical`

```bash
# Find large files
du -sh /var/lib/docker/volumes/* | sort -rh | head -10

# Prune unused Docker data
docker system prune -f

# Truncate old Prometheus data if needed (adjust retention in docker-compose.yml)
# Reduce --storage.tsdb.retention.time=15d and restart prometheus
docker compose restart prometheus
```

### RB-012: PostgreSQL Connection Exhaustion

**Alert:** `PostgresHighConnections`

```bash
# Check active connections
docker compose exec postgres psql -U indexer -d stellar_escrow \
  -c "SELECT count(*), state FROM pg_stat_activity GROUP BY state;"

# Kill idle connections older than 10 minutes
docker compose exec postgres psql -U indexer -d stellar_escrow \
  -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity
      WHERE state = 'idle' AND query_start < NOW() - INTERVAL '10 minutes';"
```

### RB-013: Redis Down / Low Hit Rate

**Alert:** `RedisDown` / `RedisLowHitRate`

See [RB-005](#rb-005-redis-cache-failure). For low hit rate, check TTL configuration in `indexer/src/cache.rs` and consider increasing cache TTLs for read-heavy endpoints.

---

*© 2026 StellarEscrow*
