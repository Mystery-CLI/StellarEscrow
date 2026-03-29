#!/usr/bin/env bash
# scripts/backup.sh
# PostgreSQL backup with S3 upload, retention enforcement, and metric reporting.
# Cron: 0 2 * * * /opt/stellarescrow/scripts/backup.sh >> /var/log/stellar-backup.log 2>&1

set -euo pipefail

# ── Config (override via environment) ────────────────────────────────────────
DB_URL="${DATABASE_URL:-postgres://indexer:password@localhost:5432/stellar_escrow}"
BACKUP_DIR="${BACKUP_DIR:-/var/backups/stellarescrow}"
S3_BUCKET="${S3_BUCKET:-}"                  # e.g. s3://stellarescrow-backups
RETENTION_DAYS="${RETENTION_DAYS:-30}"
API_BASE="${API_BASE_URL:-http://localhost:3000}"
ALERT_WEBHOOK="${BACKUP_ALERT_WEBHOOK:-}"

TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ)
BACKUP_FILE="${BACKUP_DIR}/stellar_escrow_${TIMESTAMP}.sql.gz"

log()   { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }
alert() {
  [[ -z "$ALERT_WEBHOOK" ]] && return
  curl -sf -X POST "$ALERT_WEBHOOK" \
    -H "Content-Type: application/json" \
    -d "{\"text\":\"$*\"}" || true
}
metric() {
  curl -sf -X POST "${API_BASE}/api/metrics" \
    -H "Content-Type: application/json" \
    -d "$1" || true
}

# ── Backup ────────────────────────────────────────────────────────────────────
mkdir -p "$BACKUP_DIR"

log "Starting backup → ${BACKUP_FILE}"
pg_dump "$DB_URL" | gzip > "$BACKUP_FILE"
SIZE=$(du -sh "$BACKUP_FILE" | cut -f1)
log "Backup complete (${SIZE})"

# ── Upload to S3 ──────────────────────────────────────────────────────────────
if [[ -n "$S3_BUCKET" ]]; then
  log "Uploading to ${S3_BUCKET}..."
  aws s3 cp "$BACKUP_FILE" "${S3_BUCKET}/$(basename "$BACKUP_FILE")" \
    --storage-class STANDARD_IA
  log "Upload complete."
fi

# ── Retention: remove local backups older than RETENTION_DAYS ─────────────────
find "$BACKUP_DIR" -name "stellar_escrow_*.sql.gz" \
  -mtime "+${RETENTION_DAYS}" -delete
log "Pruned backups older than ${RETENTION_DAYS} days."

# ── Report metric ─────────────────────────────────────────────────────────────
metric "{\"source\":\"backup\",\"type\":\"backup_success\",\"file\":\"$(basename "$BACKUP_FILE")\",\"ts\":$(date +%s)}"
alert "✅ StellarEscrow backup complete: $(basename "$BACKUP_FILE") (${SIZE})"

# Consumed by indexer BackupService (`run_backup_script`) to persist artifact path
echo "BACKUP_LOCATION=${BACKUP_FILE}"
