# StellarEscrow — Performance Optimization

## Bottleneck Analysis

Run the analysis script to identify current bottlenecks:

```bash
./scripts/perf-analyze.sh          # human-readable
./scripts/perf-analyze.sh --json   # JSON output for dashboards
```

The script checks:
1. **Slow queries** — `pg_stat_statements` top 10 by mean execution time
2. **Index usage** — tables with low index-scan ratio (candidates for new indexes)
3. **API latency** — live data from `/health/metrics`
4. **APM bottlenecks** — live data from `/performance/bottlenecks` (slow queries + index usage + cache stats)
5. **Container resources** — CPU/memory via `docker stats`
6. **Redis hit rate** — `keyspace_hits` vs `keyspace_misses`

### Live Bottleneck API

`GET /performance/bottlenecks` returns a JSON report combining:
- Top 10 slow queries from `pg_stat_statements`
- Tables with low index-scan ratio
- Current cache hit/miss stats

```bash
curl http://localhost:3000/performance/bottlenecks | jq .
```

### Slow Query Threshold

Queries exceeding **100ms** are automatically logged to the `slow_query_log` table.
Query the table to find recurring slow patterns:

```sql
SELECT query_hash, query_text, round(avg(duration_ms)::numeric, 1) AS avg_ms, count(*) AS hits
FROM slow_query_log
WHERE recorded_at > NOW() - INTERVAL '24 hours'
GROUP BY query_hash, query_text
ORDER BY avg_ms DESC
LIMIT 20;
```

---

## Caching Strategy

### Redis API Cache (`indexer/src/cache.rs`)

| Endpoint pattern | TTL | Config key | Rationale |
|-----------------|-----|-----------|-----------|
| `GET /events*` | 10s | `events_ttl_secs` | High-frequency reads; Stellar ledger closes every ~5s |
| `GET /search*` | 30s | `search_ttl_secs` | Search results change infrequently |
| `GET /stats` | 60s | `stats_ttl_secs` | Aggregate — expensive to compute |
| `GET /analytics/dashboard` | 60s | `analytics_ttl_secs` | Heavy aggregation query |
| `POST /events/replay` | no cache | — | Mutating |

**Activate:** set `redis_url` in `config.toml` or `STELLAR_ESCROW__CACHE__REDIS_URL` env var.  
**Fallback:** if Redis is unavailable, all requests hit Postgres directly — no errors.

All TTLs are configurable in `indexer/config.toml` under `[cache]`.

### Client-Side Cache (`frontend/performance.js`)

`cachedFetch()` provides an in-memory TTL cache (default 30s) for API responses.  
Call `invalidateCache(url)` after any write operation to keep the UI consistent.

### CDN / HTTP Cache (`cdn-headers.nginx.conf`)

| Asset type | Cache-Control |
|-----------|--------------|
| Hashed JS/CSS | `immutable, max-age=31536000` |
| Images/fonts | `max-age=86400, stale-while-revalidate=3600` |
| HTML/JSON | `max-age=300` |
| Service worker | `no-store` |

---

## Resource Allocation

### Docker Resource Limits

Applied in `docker-compose.yml` for all application services:

| Service | CPU limit | Memory limit | CPU reservation | Memory reservation |
|---------|-----------|-------------|-----------------|-------------------|
| indexer | 1.0 | 512M | 0.25 | 128M |
| api | 0.5 | 256M | 0.1 | 64M |

Adjust limits based on observed usage from `docker stats` or the Grafana infrastructure dashboard.

### Database Connection Pool

Configured in `indexer/config.toml`:

```toml
[database]
max_connections = 10   # increase for high-concurrency deployments
min_connections = 2    # keep warm connections ready
```

**Rule of thumb:** `max_connections = (2 × CPU cores) + effective_spindle_count`  
For a 4-core host: set `max_connections = 10–15`.

Monitor pool saturation via `GET /performance/bottlenecks` → `index_usage` or the Grafana PostgreSQL connections panel.

### Nginx (`performance.nginx.conf`)

- `worker_processes auto` — one worker per CPU core
- `worker_connections 4096` — handles ~4k concurrent connections per worker
- `keepalive 32` on the upstream pool — reuses TCP connections to the Rust indexer
- `gzip` on for JS/CSS/JSON — typically 60–80% size reduction

### Docker Resource Limits

Add to `docker-compose.yml` services for production:

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

## Performance Monitoring

Metrics are collected at three layers:

| Layer | Mechanism | Endpoint |
|-------|-----------|---------|
| Infrastructure | `HealthMonitor` (Rust) | `GET /health/metrics` |
| APM (request-level) | `PerformanceService` (Rust) | `GET /performance/dashboard` |
| APM alerts | `PerformanceService` alert rules | `GET /performance/alerts` |
| APM history | Hourly rollup (materialized view) | `GET /performance/history` |
| Prometheus scrape | `MonitoringService` + APM metrics | `GET /metrics` |
| Web Vitals | `observeWebVitals()` (JS) | beacons → `POST /api/metrics` |
| CDN | `observeCdnPerformance()` (JS) | beacons → `POST /api/metrics` |
| SSL | `monitorTlsConnection()` (JS) | beacons → `POST /api/metrics` |

### APM Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /performance/dashboard` | Full APM snapshot: per-route stats, overall P95/P99, active alerts |
| `GET /performance/alerts` | Active performance alerts only |
| `GET /performance/history` | Hourly rollup from `perf_metrics_hourly` (last 24h) |
| `POST /performance/record` | Ingest a sample `{ route, method, status, duration_ms }` |
| `GET /metrics` | Prometheus text format (monitoring + APM metrics) |

### Grafana Dashboards

| Dashboard | UID | Description |
|-----------|-----|-------------|
| StellarEscrow Platform | `stellar-escrow-main` | Business metrics (trades, compliance, fraud) |
| APM — Performance | `stellar-escrow-apm` | Latency (avg/P95/P99), error rate, throughput, DB query time |
| Code Quality | `code-quality` | CI lint/security metrics |

### Alert Rules

| File | Group | Alerts |
|------|-------|--------|
| `alert_rules.yml` | `stellar_escrow_platform` | Error rate, disputes, fraud |
| `alert_rules_security.yml` | `stellar_escrow_compliance` | AML, compliance blocks |
| `alert_rules_performance.yml` | `stellar_escrow_performance` | Latency (avg/P95), error rate, DB queries, throughput |

### Key Metrics to Watch

- **TTFB** < 200ms (good), < 800ms (acceptable)
- **LCP** < 2.5s
- **Avg response time** < 500ms (warning at 500ms, critical at 2000ms)
- **P95 response time** < 1000ms (warning at 1000ms, critical at 5000ms)
- **DB query mean** < 10ms for hot paths (`get_events`, `search_trades`)
- **Redis hit rate** > 80% under normal load
- **Indexer memory** < 256MB under normal load

---

## Quick Wins Checklist

- [ ] Enable Redis (`redis_url` in config) — eliminates repeat DB hits for read-heavy endpoints
- [ ] Enable `pg_stat_statements` extension — required for slow query analysis (migration `20260329000001_perf_optimization.sql` does this)
- [ ] Set `max_connections` based on actual CPU count
- [ ] Include `performance.nginx.conf` in nginx http block
- [ ] Add `STELLAR_ESCROW__CACHE__REDIS_URL` to docker-compose environment
- [ ] Run `perf-analyze.sh` weekly and track trends
- [ ] Review `GET /performance/bottlenecks` after each deploy
- [ ] Check `slow_query_log` table weekly for recurring slow patterns
- [ ] Tune Docker resource limits based on `docker stats` observations
