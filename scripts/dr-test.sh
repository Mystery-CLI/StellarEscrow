#!/usr/bin/env bash
# scripts/dr-test.sh
# Disaster recovery test: spins up an isolated Postgres container,
# restores the latest backup into it, runs verification queries,
# and reports pass/fail to /api/metrics.
#
# Run monthly or after every deployment:
#   ./scripts/dr-test.sh [backup-file]

set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-/var/backups/stellarescrow}"
S3_BUCKET="${S3_BUCKET:-}"
API_BASE="${API_BASE_URL:-http://localhost:3000}"
ALERT_WEBHOOK="${BACKUP_ALERT_WEBHOOK:-}"
TEST_CONTAINER="stellar_dr_test_$$"
TEST_PORT=15432

log()   { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }
pass()  { log "✅ PASS: $*"; }
fail()  { log "❌ FAIL: $*"; FAILED=1; }
alert() {
  [[ -z "$ALERT_WEBHOOK" ]] && return
  curl -sf -X POST "$ALERT_WEBHOOK" -H "Content-Type: application/json" -d "{\"text\":\"$*\"}" || true
}
metric(){ curl -sf -X POST "${API_BASE}/api/metrics" -H "Content-Type: application/json" -d "$1" || true; }

FAILED=0
cleanup() {
  docker rm -f "$TEST_CONTAINER" 2>/dev/null || true
  [[ -f "${TMP_BACKUP:-}" ]] && rm -f "$TMP_BACKUP"
}
trap cleanup EXIT

# ── Resolve latest backup ─────────────────────────────────────────────────────
if [[ -n "${1:-}" ]]; then
  BACKUP_FILE="$1"
elif [[ -n "$S3_BUCKET" ]]; then
  log "Fetching latest backup from S3..."
  TMP_BACKUP="/tmp/stellar_dr_test_$$.sql.gz"
  LATEST_KEY=$(aws s3 ls "${S3_BUCKET}/" | sort | tail -1 | awk '{print $4}')
  aws s3 cp "${S3_BUCKET}/${LATEST_KEY}" "$TMP_BACKUP"
  BACKUP_FILE="$TMP_BACKUP"
else
  BACKUP_FILE=$(ls -t "${BACKUP_DIR}"/stellar_escrow_*.sql.gz 2>/dev/null | head -1)
fi

[[ -f "$BACKUP_FILE" ]] || { log "ERROR: no backup found."; exit 1; }
log "Testing restore of: $(basename "$BACKUP_FILE")"

# ── Start isolated Postgres ───────────────────────────────────────────────────
log "Starting test Postgres container..."
docker run -d --name "$TEST_CONTAINER" \
  -e POSTGRES_DB=stellar_escrow \
  -e POSTGRES_USER=indexer \
  -e POSTGRES_PASSWORD=testpass \
  -p "${TEST_PORT}:5432" \
  postgres:15 > /dev/null

TEST_DB="postgres://indexer:testpass@localhost:${TEST_PORT}/stellar_escrow"

# Wait for Postgres to be ready
for i in $(seq 1 20); do
  pg_isready -h localhost -p "$TEST_PORT" -U indexer &>/dev/null && break
  sleep 1
done

# ── Restore ───────────────────────────────────────────────────────────────────
log "Restoring backup..."
gunzip -c "$BACKUP_FILE" | psql "$TEST_DB" > /dev/null
pass "Backup restored without errors."

# ── Verification queries ──────────────────────────────────────────────────────
TRADE_COUNT=$(psql "$TEST_DB" -tAc "SELECT COUNT(*) FROM trades;" 2>/dev/null || echo 0)
EVENT_COUNT=$(psql "$TEST_DB" -tAc "SELECT COUNT(*) FROM events;"  2>/dev/null || echo 0)

[[ "$TRADE_COUNT" -gt 0 ]] && pass "trades table: ${TRADE_COUNT} rows" || fail "trades table is empty"
[[ "$EVENT_COUNT" -gt 0 ]] && pass "events table: ${EVENT_COUNT} rows" || fail "events table is empty"

# Schema integrity check
TABLES=$(psql "$TEST_DB" -tAc "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public';" 2>/dev/null || echo 0)
[[ "$TABLES" -ge 3 ]] && pass "Schema intact: ${TABLES} tables" || fail "Schema incomplete: only ${TABLES} tables"

# ── Report ────────────────────────────────────────────────────────────────────
STATUS=$([[ $FAILED -eq 0 ]] && echo "pass" || echo "fail")
metric "{\"source\":\"dr_test\",\"type\":\"recovery_test\",\"status\":\"${STATUS}\",\"trades\":${TRADE_COUNT},\"events\":${EVENT_COUNT},\"ts\":$(date +%s)}"

if [[ $FAILED -eq 0 ]]; then
  alert "✅ StellarEscrow DR test PASSED — backup $(basename "$BACKUP_FILE") restored successfully."
  log "DR test PASSED."
else
  alert "🚨 StellarEscrow DR test FAILED — check logs."
  log "DR test FAILED."
  exit 1
fi
