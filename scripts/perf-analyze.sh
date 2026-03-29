#!/usr/bin/env bash
# scripts/perf-analyze.sh
# Analyze StellarEscrow performance bottlenecks:
#   - Slow PostgreSQL queries
#   - API endpoint latency (from /health/metrics)
#   - Docker container resource usage
#   - Redis cache hit rate
#
# Usage: ./scripts/perf-analyze.sh [--json]

set -euo pipefail

API_BASE="${API_BASE_URL:-http://localhost:3000}"
DB_URL="${DATABASE_URL:-postgres://indexer:password@localhost:5432/stellar_escrow}"
JSON_OUTPUT=0
[[ "${1:-}" == "--json" ]] && JSON_OUTPUT=1

log() { [[ $JSON_OUTPUT -eq 0 ]] && echo "$*" || true; }
sep() { log "────────────────────────────────────────"; }

declare -A REPORT

# ── 1. Slow queries (pg_stat_statements) ─────────────────────────────────────
log ""
log "## Slow Queries (top 10 by mean exec time)"
sep
SLOW_QUERIES=$(psql "$DB_URL" -tA --csv -c "
  SELECT query, calls, round(mean_exec_time::numeric,2) AS mean_ms,
         round(total_exec_time::numeric,2) AS total_ms,
         round(stddev_exec_time::numeric,2) AS stddev_ms
  FROM pg_stat_statements
  ORDER BY mean_exec_time DESC
  LIMIT 10;
" 2>/dev/null || echo "pg_stat_statements not available")
log "$SLOW_QUERIES"
REPORT[slow_queries]="$SLOW_QUERIES"

# ── 2. Table bloat / index usage ──────────────────────────────────────────────
log ""
log "## Index Usage (tables with low index scan ratio)"
sep
INDEX_USAGE=$(psql "$DB_URL" -tA --csv -c "
  SELECT relname AS table,
         seq_scan, idx_scan,
         CASE WHEN seq_scan + idx_scan = 0 THEN 0
              ELSE round(100.0 * idx_scan / (seq_scan + idx_scan), 1)
         END AS idx_pct
  FROM pg_stat_user_tables
  ORDER BY idx_pct ASC
  LIMIT 10;
" 2>/dev/null || echo "unavailable")
log "$INDEX_USAGE"
REPORT[index_usage]="$INDEX_USAGE"

# ── 3. API latency from /health/metrics ───────────────────────────────────────
log ""
log "## API Health Metrics"
sep
API_METRICS=$(curl -sf "${API_BASE}/health/metrics" 2>/dev/null || echo "unavailable")
log "$API_METRICS"
REPORT[api_metrics]="$API_METRICS"

# ── 4. Docker container resource usage ───────────────────────────────────────
log ""
log "## Container Resource Usage"
sep
DOCKER_STATS=$(docker stats --no-stream --format \
  "{{.Name}}\tCPU={{.CPUPerc}}\tMEM={{.MemUsage}}\tNET={{.NetIO}}" \
  2>/dev/null || echo "Docker not available")
log "$DOCKER_STATS"
REPORT[docker_stats]="$DOCKER_STATS"

# ── 5. Redis cache hit rate ───────────────────────────────────────────────────
log ""
log "## Redis Cache Stats"
sep
REDIS_URL="${REDIS_URL:-redis://localhost:6379}"
REDIS_STATS=$(redis-cli -u "$REDIS_URL" INFO stats 2>/dev/null \
  | grep -E "keyspace_hits|keyspace_misses|instantaneous_ops_per_sec" \
  || echo "Redis not available")
log "$REDIS_STATS"
REPORT[redis_stats]="$REDIS_STATS"

# ── JSON output ───────────────────────────────────────────────────────────────
if [[ $JSON_OUTPUT -eq 1 ]]; then
  python3 -c "
import json, sys
data = {}
for k, v in [$(for k in "${!REPORT[@]}"; do echo "('$k', '''${REPORT[$k]}'''),"; done)]:
    data[k] = v
print(json.dumps(data, indent=2))
"
fi

log ""
log "Analysis complete. Review slow queries and low index-scan ratios for optimization targets."
