#!/usr/bin/env bash
# scripts/secrets-rotate.sh
# Rotate application secrets in AWS Secrets Manager and sync to Kubernetes.
#
# Usage:
#   ./scripts/secrets-rotate.sh [--dry-run]
#
# Required env:
#   AWS_REGION          AWS region (default: us-east-1)
#   K8S_NAMESPACE       Kubernetes namespace (default: stellar-escrow)
#   ALERT_WEBHOOK       Slack/webhook URL for rotation notifications

set -euo pipefail

DRY_RUN=false
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=true

AWS_REGION="${AWS_REGION:-us-east-1}"
K8S_NAMESPACE="${K8S_NAMESPACE:-stellar-escrow}"
ALERT_WEBHOOK="${ALERT_WEBHOOK:-}"

log()   { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }
alert() {
  [[ -z "$ALERT_WEBHOOK" ]] && return
  curl -sf -X POST "$ALERT_WEBHOOK" \
    -H "Content-Type: application/json" \
    -d "{\"text\":\"$*\"}" || true
}

rotate_secret() {
  local secret_id="$1"
  log "Rotating secret: ${secret_id}"
  if [[ "$DRY_RUN" == "true" ]]; then
    log "[DRY RUN] Would rotate: ${secret_id}"
    return
  fi
  aws secretsmanager rotate-secret \
    --secret-id "$secret_id" \
    --region "$AWS_REGION"
  log "Rotated: ${secret_id}"
}

sync_k8s_secret() {
  local secret_id="$1"
  local k8s_secret_name="$2"
  log "Syncing ${secret_id} → k8s secret ${k8s_secret_name}"
  if [[ "$DRY_RUN" == "true" ]]; then
    log "[DRY RUN] Would sync to k8s"
    return
  fi
  # Force CSI driver to re-sync by restarting pods that mount the secret
  kubectl rollout restart deployment \
    -n "$K8S_NAMESPACE" \
    -l "secrets-store.csi.k8s.io/managed=true" || true
}

# ── Rotate secrets ────────────────────────────────────────────────────────────
log "Starting secrets rotation (dry_run=${DRY_RUN})"

rotate_secret "stellar-escrow/production/database"
rotate_secret "stellar-escrow/production/api-keys"

sync_k8s_secret "stellar-escrow/production/database" "stellar-escrow-secrets"
sync_k8s_secret "stellar-escrow/production/api-keys" "stellar-escrow-secrets"

alert "🔑 StellarEscrow: secrets rotation complete (dry_run=${DRY_RUN}) — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
log "Rotation complete."
