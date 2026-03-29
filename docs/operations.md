# Operational Procedures

> StellarEscrow — Day-to-Day Platform Management  
> Audience: On-Call Engineers, Platform Team, DevOps  
> Review Cadence: Quarterly or after major incidents

## Table of Contents

- [Daily Health Check](#daily-health-check)
- [Scheduled Maintenance](#scheduled-maintenance)
- [Backup Operations](#backup-operations)
- [Secrets Management](#secrets-management)
- [Configuration Updates](#configuration-updates)
- [Scaling](#scaling)
- [Monitoring & Alerting](#monitoring--alerting)
- [Change Management](#change-management)

---

## Daily Health Check

Run at the start of every on-call shift:

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

# 5. Check indexer lag
curl -s http://localhost:3000/health | python3 -m json.tool
```

---

## Scheduled Maintenance

| Schedule | Task | Script | Log |
|----------|------|--------|-----|
| Daily @ 02:00 UTC | Database backup to S3 | `scripts/backup.sh` | `/var/log/stellar-backup.log` |
| Monthly @ 04:00 UTC (1st) | Disaster recovery test | `scripts/dr-test.sh` | `/var/log/stellar-dr-test.log` |
| Twice daily (00:00, 12:00) | TLS certificate renewal | `scripts/renew-certs.sh` | `/var/log/cert-renewal.log` |
| Weekly (Sunday 03:00) | Performance analysis | `scripts/perf-analyze.sh` | `/var/log/perf-report.log` |
| Quarterly | API key rotation | Manual (see [Secrets Management](#secrets-management)) | Change log |

```bash
# Full crontab
0 2 * * *    /opt/stellarescrow/scripts/backup.sh >> /var/log/stellar-backup.log 2>&1
0 4 1 * *    /opt/stellarescrow/scripts/dr-test.sh >> /var/log/stellar-dr-test.log 2>&1
0 0,12 * * * /opt/stellarescrow/scripts/renew-certs.sh >> /var/log/cert-renewal.log 2>&1
0 3 * * 0    /opt/stellarescrow/scripts/perf-analyze.sh >> /var/log/perf-report.log 2>&1
```

---

## Backup Operations

### Automated Daily Backup

The `backup` Docker service runs `scripts/backup.sh` daily at 02:00 UTC. It performs `pg_dump`, compresses with gzip, uploads to S3, and sends webhook notifications.

```bash
# Trigger a manual backup immediately
docker compose run --rm backup bash /scripts/backup.sh

# Verify the latest backup
aws s3 ls s3://stellarescrow-backups/ | sort | tail -5
aws s3 cp s3://stellarescrow-backups/stellar_escrow_<TIMESTAMP>.sql.gz /tmp/test.sql.gz
gzip -t /tmp/test.sql.gz && echo "Backup valid"
```

### Monthly DR Test

```bash
# Test with latest backup (auto-detected)
./scripts/dr-test.sh

# Test with a specific backup
./scripts/dr-test.sh /var/backups/stellarescrow/stellar_escrow_<TIMESTAMP>.sql.gz

# Test with S3 backup
S3_BUCKET=s3://stellarescrow-backups ./scripts/dr-test.sh
```

The DR test spins up an isolated Postgres container, restores the backup, verifies row counts in `trades` and `events`, and reports `pass`/`fail` to `/api/metrics`.

---

## Secrets Management

### Rotation Schedule

| Secret | Frequency | Procedure |
|--------|-----------|-----------|
| `POSTGRES_PASSWORD` | Annually or on breach | Update `.env`, restart postgres, update `DATABASE_URL`, restart all services |
| `REDIS_PASSWORD` | Annually or on breach | Update `.env`, restart redis; services reconnect automatically |
| `API_KEYS` | Quarterly | See below |
| `ADMIN_KEYS` | Quarterly | Same as API key rotation |
| `GRAFANA_ADMIN_PASSWORD` | Quarterly | Update via Grafana UI, then update `.env` |
| S3 credentials | Per IAM policy | Update AWS credentials/IAM role |

### API Key Rotation (Quarterly)

> Coordinate with all API consumers before rotating. Provide **1 week notice**.

```bash
# 1. Generate new keys
openssl rand -hex 32   # Run once per key needed

# 2. Add new keys alongside old ones (transition window)
# Edit .env:
API_KEYS=old-key,new-key-1,new-key-2
docker compose restart api

# 3. After all consumers have switched, remove old keys
API_KEYS=new-key-1,new-key-2
docker compose restart api
```

### TLS Certificate Management

```bash
# Check certificate expiry
echo | openssl s_client -connect stellarescrow.app:443 2>/dev/null \
  | openssl x509 -noout -dates

# Force renewal if < 30 days remaining
docker compose run --rm certbot renew --force-renewal
docker compose exec nginx nginx -s reload
```

---

## Configuration Updates

### Updating Environment Variables

```bash
# 1. Edit .env
nano .env

# 2. Restart only the affected service(s)
docker compose restart <service>

# 3. Verify health
docker compose ps
curl -sf http://localhost:<port>/health
```

### Updating `indexer/config.toml`

The indexer reads `config.toml` at startup. After editing:

```bash
docker compose restart indexer
docker compose logs -f indexer   # Confirm no config errors
```

### Updating Nginx Config

After editing `ssl.nginx.conf`, `cdn-headers.nginx.conf`, or `performance.nginx.conf`:

```bash
# Test config before applying
docker compose exec nginx nginx -t

# Apply without downtime
docker compose exec nginx nginx -s reload
```

---

## Scaling

### Vertical Scaling (Single Host)

Increase Docker resource limits in `docker-compose.yml`:

```yaml
# Under the indexer or api service:
deploy:
  resources:
    limits:
      memory: 512M
      cpus: "1.0"
```

Then: `docker compose up -d --no-deps <service>`

### Database Connection Pool

Edit `indexer/config.toml`:

```toml
[database]
max_connections = 20   # Rule of thumb: (2 × CPU cores) + spindle count
min_connections = 4
```

Restart the indexer after changes.

### Redis Memory

Edit `docker-compose.yml` under the `redis` service:

```yaml
command: >
  redis-server
  --maxmemory 512mb   # Increase from default 256mb
  --maxmemory-policy allkeys-lru
  ...
```

### Kubernetes (Production Scale)

Kubernetes manifests are in `k8s/`. Apply with Kustomize:

```bash
# Deploy all services
kubectl apply -k k8s/

# Scale a deployment
kubectl scale deployment api --replicas=3 -n stellar-escrow

# Check rollout status
kubectl rollout status deployment/api -n stellar-escrow
```

Terraform infrastructure definitions are in `infra/terraform/`. Environment-specific variables are in `infra/terraform/environments/`.

---

## Monitoring & Alerting

### Grafana Dashboards

Access at `http://localhost:3002` (internal) or `https://grafana.stellarescrow.app`.  
Login: `admin` / `GRAFANA_ADMIN_PASSWORD` from `.env`.

| Dashboard | Key Panels |
|-----------|-----------|
| Infrastructure Overview | Service status, request rate, error rate, resource usage |
| Indexer Performance | Ledger lag, events/sec, DB write latency |
| API Latency | P50/P95/P99 by endpoint, error breakdown |
| Database Metrics | Query latency, connection count, cache hit rate |
| Trade Analytics | Volume, success rate, dispute rate, fees |
| Web Vitals | TTFB, LCP, FID, CLS by page |

### Alert Response SLAs

| Alert | Severity | Response SLA | Runbook |
|-------|----------|-------------|---------|
| `ServiceDown` | P1 | 15 min | [RB-001 – RB-005](runbooks/troubleshooting.md) |
| `IndexerLag > 5 min` | P2 | 1 hour | [RB-002](runbooks/troubleshooting.md#rb-002-indexer-not-processing-events) |
| `TLSCertExpiringSoon` | P2 | 4 hours | [RB-004](runbooks/troubleshooting.md#rb-004-nginx--tls-issues) |
| `DBQueryLatency > 50ms` | P3 | 4 hours | [RB-006](runbooks/troubleshooting.md#rb-006-high-database-query-latency) |
| `RedisHitRate < 60%` | P3 | 4 hours | [RB-005](runbooks/troubleshooting.md#rb-005-redis-cache-failure) |
| `BackupFailed` | P2 | 1 hour | [Backup Operations](#backup-operations) |
| `DiskUsage > 85%` | P2 | 2 hours | [Scaling](#scaling) |

---

## Change Management

| Change Type | Approval | Process |
|-------------|----------|---------|
| Config-only (env vars) | On-call lead | Update `.env`, rolling restart, verify health |
| Application update | On-call lead + peer review | PR review, staging validation, rolling deploy |
| Database migration | Platform lead | Script review, staging test, rollback plan |
| New contract deployment | Platform lead + CTO | Audit, testnet validation, mainnet multisig |
| Infrastructure change | Platform lead | Review, staging deploy, production with monitoring |

### Post-Incident Review

After every P1 or P2 incident, conduct a blameless review within 48 hours. Document:

- Timeline (UTC timestamps)
- Root cause
- Impact (duration, affected users, financial impact)
- Immediate remediation steps
- Long-term corrective actions with owners and due dates
- Detection gap — could monitoring have caught this earlier?

---

*See also: [Architecture](architecture.md) | [Deployment](deployment.md) | [Troubleshooting](runbooks/troubleshooting.md)*
