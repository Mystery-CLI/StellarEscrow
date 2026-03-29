# Troubleshooting Runbooks

> StellarEscrow — Incident Response & Recovery Procedures  
> Audience: On-Call Engineers, SRE Team  
> Format: Problem → Diagnosis → Resolution  
> **RTO Target:** ≤ 2 hours | **RPO Target:** ≤ 24 hours

## Quick Reference

| Resource | URL |
|----------|-----|
| Stellar Network Status | https://status.stellar.org |
| Grafana | http://localhost:3002 |
| Alertmanager | http://localhost:9093 |
| Prometheus | http://localhost:9090 |

## Table of Contents

- [Incident Classification](#incident-classification)
- [First-Responder Checklist](#first-responder-checklist)
- [RB-001: PostgreSQL Failure](#rb-001-postgresql-failure)
- [RB-002: Indexer Not Processing Events](#rb-002-indexer-not-processing-events)
- [RB-003: API Gateway 5xx Errors](#rb-003-api-gateway-5xx-errors)
- [RB-004: Nginx / TLS Issues](#rb-004-nginx--tls-issues)
- [RB-005: Redis Cache Failure](#rb-005-redis-cache-failure)
- [RB-006: High Database Query Latency](#rb-006-high-database-query-latency)
- [RB-007: Stellar Network Degraded](#rb-007-stellar-network-degraded)
- [RB-008: Database Restore from Backup](#rb-008-database-restore-from-backup)
- [RB-009: Complete Host Rebuild](#rb-009-complete-host-rebuild)
- [Escalation Matrix](#escalation-matrix)

---

## Incident Classification

| Severity | Description | Response Time | Examples |
|----------|-------------|---------------|---------|
| **P1 — Critical** | Complete outage; active fund loss risk | 15 minutes | Nginx down, DB unrecoverable, contract exploit |
| **P2 — High** | Major feature broken; many users affected | 1 hour | Indexer stopped, API all-5xx, TLS expired |
| **P3 — Medium** | Degraded performance or partial outage | 4 hours | High latency, Redis down, cache miss spike |
| **P4 — Low** | Minor issue; workaround available | Next business day | Grafana login issue, log gap, slow CSV export |

---

## First-Responder Checklist

Run **in order** for any unclassified incident:

```bash
# 1. Check overall service health
docker compose ps

# 2. Check web health endpoint
curl -I https://stellarescrow.app/health

# 3. Review recent logs (last 30 minutes)
docker compose logs --since 30m --tail 100

# 4. Check Stellar network status
#    → https://status.stellar.org

# 5. Check active Prometheus alerts
#    → http://localhost:9093

# 6. Check disk space
df -h

# 7. Check memory
free -h
```

Identify the failing component and jump to the relevant runbook below.

---

## RB-001: PostgreSQL Failure

**Problem:** Database is unreachable or unhealthy, causing downstream service failures.

**Symptoms:** Indexer cannot connect to database; API returns HTTP 500 or 503; Grafana shows "DB connection failed" alert.

### Diagnosis

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

**Common error codes:**

| Log Pattern | Likely Cause |
|-------------|-------------|
| `FATAL: password authentication failed` | Wrong `POSTGRES_PASSWORD` in `.env` |
| `FATAL: could not open file ... No space left` | Disk full |
| `PANIC: could not write to file ... Input/output error` | Disk I/O failure or corruption |
| `connection refused` | Container stopped or not yet healthy |

### Resolution

**1. Restart if stopped or unhealthy**

```bash
docker compose restart postgres
# Wait 30 seconds, then verify:
docker compose ps postgres   # Should show "healthy"
```

**2. If restart fails — check for data corruption**

```bash
docker compose logs postgres | grep -i "FATAL\|PANIC\|corrupt"
# If corruption detected → proceed to RB-008
```

**3. If disk is full — clear Docker build cache**

```bash
docker system prune -f
# If still insufficient, expand disk via cloud provider console, then restart
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

## RB-002: Indexer Not Processing Events

**Problem:** The indexer has stopped polling Stellar or writing events to the database.

**Symptoms:** Trades confirmed on-chain but not appearing in UI; stale trade status; Grafana "Indexer lag" alert fires.

### Diagnosis

```bash
# Check indexer health
curl http://localhost:3000/health

# Check for errors
docker compose logs --tail 100 indexer | grep -i "error\|WARN\|panic"

# Check Stellar network status
# → https://status.stellar.org

# Check last processed ledger
docker compose exec postgres psql -U indexer -d stellar_escrow \
  -c "SELECT MAX(ledger_sequence) FROM events;"
```

**Common error codes:**

| Log Pattern | Likely Cause |
|-------------|-------------|
| `failed to fetch ledger` | Stellar network degraded → see RB-007 |
| `connection refused` (DB) | PostgreSQL down → see RB-001 |
| `OOMKilled` / exit code 137 | Out of memory |
| `thread 'main' panicked` | Bug or config error; check full stack trace |

### Resolution

**1. If Stellar network is degraded — wait**

> Do **not** restart the indexer during an active Stellar network incident. It retries with exponential backoff and will resume automatically with no data loss.

**2. If indexer is OOM-killed — increase memory limit**

```bash
# Confirm OOM kill (exit code 137)
docker inspect stellar-escrow-indexer-1 | grep "ExitCode"

# Add to docker-compose.yml under the indexer service:
# deploy:
#   resources:
#     limits:
#       memory: 512M
docker compose up -d --no-deps indexer
```

**3. If logs show database connection errors — see RB-001**

**4. If logs show panics — rebuild and redeploy**

```bash
docker compose build indexer
docker compose up -d --no-deps indexer
```

**5. Trigger event replay if ledger gaps exist**

```bash
curl -X POST http://localhost:3000/events/replay \
  -H "Authorization: Bearer $ADMIN_KEY" \
  -d '{"from_ledger": <START>, "to_ledger": <END>}'
```

---

## RB-003: API Gateway 5xx Errors

**Problem:** The API gateway is returning server errors to clients.

**Symptoms:** Frontend displays errors; POST `/api/*` requests fail; API health endpoint returns non-200.

### Diagnosis

```bash
# Health check
curl -v http://localhost:4000/health

# Check logs for error patterns
docker compose logs --tail 100 api | grep -E "Error|5[0-9]{2}"

# Verify dependencies are reachable from within the API container
docker compose exec api wget -qO- http://indexer:3000/health
```

**Common error codes:**

| HTTP Status | Likely Cause |
|-------------|-------------|
| 502 Bad Gateway | Upstream (indexer) is down |
| 503 Service Unavailable | API itself is crashing or overloaded |
| 500 Internal Server Error | Unhandled exception; check logs for stack trace |

### Resolution

**1. Restart the API**

```bash
docker compose restart api
```

**2. If API cannot reach indexer — fix indexer first**

See RB-002, then restart API.

**3. If API cannot reach database — see RB-001**

**4. If restart loop — check environment variables**

```bash
docker compose config api   # Verify all required env vars are present
# Ensure API_KEYS, ADMIN_KEYS, DATABASE_URL, INDEXER_URL are set
```

---

## RB-004: Nginx / TLS Issues

**Problem:** HTTPS is unreachable or returning certificate errors.

**Symptoms:** HTTPS site unreachable; SSL certificate error in browser; HTTP 502 Bad Gateway.

### Diagnosis

```bash
# Test nginx config syntax
docker compose exec nginx nginx -t

# Check certificate validity and expiry
echo | openssl s_client -connect stellarescrow.app:443 2>/dev/null \
  | openssl x509 -noout -dates

# Check upstream health (502 usually means a backend is down)
curl http://localhost:3000/health   # indexer
curl http://localhost:3001/         # client
```

**Common error codes:**

| Symptom | Likely Cause |
|---------|-------------|
| `502 Bad Gateway` | Backend service (indexer or client) is down |
| `SSL_ERROR_RX_RECORD_TOO_LONG` | HTTP traffic hitting HTTPS port |
| `certificate has expired` | Certbot renewal failed |
| `nginx: [emerg] unknown directive` | Config syntax error |

### Resolution

**1. HTTP 502 — fix the upstream service first**

Resolve the failing backend using RB-001 through RB-003. Nginx recovers automatically once backends are healthy.

**2. Expired TLS certificate — renew manually**

```bash
docker compose run --rm certbot renew --force-renewal
docker compose exec nginx nginx -s reload
```

**3. Nginx config error — fix and reload**

```bash
# Edit the relevant config file on the host:
# ssl.nginx.conf, cdn-headers.nginx.conf, or performance.nginx.conf
docker compose exec nginx nginx -t   # Verify fix
docker compose exec nginx nginx -s reload
```

---

## RB-005: Redis Cache Failure

**Problem:** Redis is unavailable or performing poorly.

**Symptoms:** Performance degraded; all requests hitting database; Grafana shows "Redis hit rate < 60%" alert; Redis container unhealthy.

> ℹ️ **Safe to tolerate briefly:** Redis failure degrades performance but does not cause errors. The system falls back to direct PostgreSQL reads automatically.

### Diagnosis

```bash
docker compose ps redis
docker compose logs --tail 30 redis
docker compose exec redis redis-cli -a "$REDIS_PASSWORD" ping
```

### Resolution

**1. Restart Redis**

```bash
docker compose restart redis
docker compose exec redis redis-cli -a "$REDIS_PASSWORD" ping
# Expected: PONG
```

**2. Check memory usage**

```bash
docker compose exec redis redis-cli -a "$REDIS_PASSWORD" info memory \
  | grep used_memory_human
# If near maxmemory (256mb), increase in docker-compose.yml:
# --maxmemory 512mb
```

---

## RB-006: High Database Query Latency

**Problem:** PostgreSQL queries are slow, causing API timeouts and poor user experience.

**Symptoms:** API responses slow (> 800ms); Grafana "DB query mean > 50ms" alert; users report slow page loads.

### Diagnosis

```bash
# Run the bottleneck analysis script
./scripts/perf-analyze.sh

# Check for long-running queries
docker compose exec postgres psql -U indexer -d stellar_escrow -c \
  "SELECT pid, now()-query_start AS duration, query
   FROM pg_stat_activity
   WHERE state='active'
   ORDER BY duration DESC
   LIMIT 10;"
```

### Resolution

**1. Kill blocking long-running queries**

```sql
SELECT pg_terminate_backend(<pid>);
```

**2. Enable Redis caching to reduce DB load**

```bash
docker compose exec indexer env | grep REDIS
# Ensure STELLAR_ESCROW__CACHE__REDIS_URL is set in .env
```

**3. Run VACUUM ANALYZE to reclaim bloat**

```bash
docker compose exec postgres psql -U indexer -d stellar_escrow \
  -c "VACUUM ANALYZE;"
```

**4. Check for missing indexes**

```bash
./scripts/perf-analyze.sh --json | python3 -m json.tool | grep -A5 "index_usage"
```

---

## RB-007: Stellar Network Degraded

**Problem:** The Stellar network is experiencing an outage or degraded performance.

**Symptoms:** Indexer logs show repeated "failed to fetch ledger" errors; no new events being processed; `status.stellar.org` shows an active incident.

> ❌ **Do NOT restart the indexer during an active Stellar network incident.** It will resume automatically with no data loss when the network recovers.

### Actions During Degradation

- Monitor https://status.stellar.org for recovery ETA
- Post a status update to users if degradation exceeds 15 minutes
- Existing trade data in PostgreSQL is fully preserved and consistent
- No new transactions can be submitted to the contract — this is expected behavior
- All historical trade data remains queryable via the API

### Post-Recovery Verification

```bash
# Verify indexer has caught up
docker compose logs --tail 50 indexer | grep "ledger"

# Check for gaps in event sequence
docker compose exec postgres psql -U indexer -d stellar_escrow -c \
  "SELECT MIN(ledger_sequence), MAX(ledger_sequence), COUNT(*)
   FROM events
   WHERE created_at > NOW() - INTERVAL '2 hours';"
```

---

## RB-008: Database Restore from Backup

**Problem:** Data loss or corruption requires restoring from a backup.

> ⚠️ This procedure causes downtime. Stop write services before restoring to prevent conflicts.

### Step 1: Identify the Recovery Point

```bash
# List local backups
ls -lt /var/backups/stellarescrow/ | head -10

# List S3 backups
aws s3 ls s3://stellarescrow-backups/ | sort | tail -10
```

### Step 2: Stop Write Services

```bash
docker compose stop indexer api
```

### Step 3: Run Restore Script

```bash
# From local file
./scripts/restore.sh /var/backups/stellarescrow/stellar_escrow_<TIMESTAMP>.sql.gz

# From S3
./scripts/restore.sh s3://stellarescrow-backups/stellar_escrow_<TIMESTAMP>.sql.gz
```

### Step 4: Verify Data Integrity

```bash
docker compose exec postgres psql -U indexer -d stellar_escrow \
  -c "SELECT COUNT(*) FROM trades; SELECT COUNT(*) FROM events;"
```

### Step 5: Restart Services and Verify

```bash
docker compose start indexer api
docker compose ps   # Wait for health checks to pass

curl -sf http://localhost:4000/health
curl -sf http://localhost:3000/health
```

---

## RB-009: Complete Host Rebuild

**Problem:** The production host is unrecoverable and must be rebuilt from scratch.

**Target RTO: ≤ 2 hours**

### Step 1: Provision a New Server

Meet minimum specifications from [Deployment Guide — Prerequisites](../deployment.md#prerequisites).

### Step 2: Install Docker and Clone Repository

```bash
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER && newgrp docker

git clone https://github.com/Mystery-CLI/StellarEscrow.git
cd StellarEscrow
```

### Step 3: Restore Secrets from Secrets Manager

```bash
cp .env.example .env
# Populate all required variables from your secrets manager
```

### Step 4: Start All Services

```bash
docker compose up -d
```

### Step 5: Restore Database from Latest S3 Backup

```bash
docker compose stop indexer api
./scripts/restore.sh s3://stellarescrow-backups/<latest>.sql.gz
docker compose start indexer api
```

### Step 6: Reissue TLS Certificate

```bash
docker compose run --rm certbot certonly \
  --webroot -w /var/www/certbot \
  -d stellarescrow.app \
  --agree-tos --non-interactive \
  --email ops@stellarescrow.app

docker compose exec nginx nginx -s reload
```

### Step 7: Update DNS if IP Changed

Update the A/AAAA record for `stellarescrow.app` to the new server IP. DNS propagation takes up to 5 minutes with a low TTL.

### Step 8: Run Post-Deployment Verification

See [Deployment Guide — Post-Deployment Verification](../deployment.md#post-deployment-verification).

---

## Escalation Matrix

| Situation | First Escalation | Second Escalation | External Contact |
|-----------|-----------------|-------------------|-----------------|
| P1 incident > 15 min | On-Call Lead | Platform Lead | — |
| Data loss suspected | Platform Lead (immediate) | CTO | AWS Support (if S3 issue) |
| Stellar network issue | Monitor status.stellar.org | — | https://status.stellar.org |
| Smart contract exploit | Platform Lead (immediate) | Security Team | Stellar Security Disclosure |
| Full host failure | On-Call Engineer | Platform Lead | Cloud provider support |

---

*See also: [Architecture](../architecture.md) | [Deployment](../deployment.md) | [Operations](../operations.md)*
