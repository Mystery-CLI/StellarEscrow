#!/usr/bin/env bash
# scripts/restore.sh
# Restore PostgreSQL from a backup file (local or S3).
#
# Usage:
#   ./scripts/restore.sh <backup-file-or-s3-key>
#
# Examples:
#   ./scripts/restore.sh /var/backups/stellarescrow/stellar_escrow_20260326T020000Z.sql.gz
#   ./scripts/restore.sh s3://stellarescrow-backups/stellar_escrow_20260326T020000Z.sql.gz

set -euo pipefail

BACKUP_SRC="${1:?Usage: $0 <backup-file-or-s3-key>}"
DB_URL="${DATABASE_URL:-postgres://indexer:password@localhost:5432/stellar_escrow}"
API_BASE="${API_BASE_URL:-http://localhost:3000}"
TMP_FILE="/tmp/stellar_restore_$$.sql.gz"

log()   { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }
metric(){ curl -sf -X POST "${API_BASE}/api/metrics" -H "Content-Type: application/json" -d "$1" || true; }

# ── Fetch backup ──────────────────────────────────────────────────────────────
if [[ "$BACKUP_SRC" == s3://* ]]; then
  log "Downloading from S3: ${BACKUP_SRC}"
  aws s3 cp "$BACKUP_SRC" "$TMP_FILE"
  LOCAL_FILE="$TMP_FILE"
else
  LOCAL_FILE="$BACKUP_SRC"
fi

[[ -f "$LOCAL_FILE" ]] || { log "ERROR: backup file not found: $LOCAL_FILE"; exit 1; }

# ── Confirm ───────────────────────────────────────────────────────────────────
log "WARNING: This will DROP and recreate the stellar_escrow database."
read -r -p "Type 'yes' to continue: " CONFIRM
[[ "$CONFIRM" == "yes" ]] || { log "Aborted."; exit 0; }

# ── Restore ───────────────────────────────────────────────────────────────────
log "Restoring from ${LOCAL_FILE}..."

# Terminate active connections
psql "$DB_URL" -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'stellar_escrow' AND pid <> pg_backend_pid();" || true

gunzip -c "$LOCAL_FILE" | psql "$DB_URL"
log "Restore complete."

# ── Verify ────────────────────────────────────────────────────────────────────
TRADE_COUNT=$(psql "$DB_URL" -tAc "SELECT COUNT(*) FROM trades;" 2>/dev/null || echo "unknown")
log "Verification: trades table has ${TRADE_COUNT} rows."

metric "{\"source\":\"recovery\",\"type\":\"restore_complete\",\"backup\":\"$(basename "$BACKUP_SRC")\",\"trades\":\"${TRADE_COUNT}\",\"ts\":$(date +%s)}"

# ── Cleanup ───────────────────────────────────────────────────────────────────
[[ -f "$TMP_FILE" ]] && rm -f "$TMP_FILE"
