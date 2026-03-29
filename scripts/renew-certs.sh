#!/usr/bin/env bash
# scripts/renew-certs.sh
# Automated Let's Encrypt certificate renewal with monitoring hooks.
# Run via cron: 0 0 * * * /opt/stellarescrow/scripts/renew-certs.sh >> /var/log/cert-renewal.log 2>&1

set -euo pipefail

DOMAIN="${CERT_DOMAIN:-stellarescrow.app}"
ALERT_WEBHOOK="${CERT_ALERT_WEBHOOK:-}"   # Slack/webhook URL for alerts
EXPIRY_WARN_DAYS=30

log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }

# ── Check expiry and alert if within threshold ────────────────────────────────
check_expiry() {
  local cert="/etc/letsencrypt/live/${DOMAIN}/fullchain.pem"
  [[ -f "$cert" ]] || { log "WARN: certificate not found at $cert"; return; }

  local expiry days_left
  expiry=$(openssl x509 -enddate -noout -in "$cert" | cut -d= -f2)
  days_left=$(( ( $(date -d "$expiry" +%s) - $(date +%s) ) / 86400 ))

  log "Certificate for ${DOMAIN} expires in ${days_left} days (${expiry})"

  if [[ $days_left -le $EXPIRY_WARN_DAYS && -n "$ALERT_WEBHOOK" ]]; then
    curl -sf -X POST "$ALERT_WEBHOOK" \
      -H "Content-Type: application/json" \
      -d "{\"text\":\"⚠️ StellarEscrow: SSL cert for ${DOMAIN} expires in ${days_left} days\"}" \
      || log "WARN: failed to send expiry alert"
  fi
}

# ── Renew via Certbot ─────────────────────────────────────────────────────────
renew() {
  log "Running certbot renew..."
  certbot renew \
    --webroot \
    --webroot-path /var/www/certbot \
    --deploy-hook "nginx -s reload" \
    --quiet

  log "Renewal complete."
}

# ── Report cert metrics to API ────────────────────────────────────────────────
report_metrics() {
  local cert="/etc/letsencrypt/live/${DOMAIN}/fullchain.pem"
  [[ -f "$cert" ]] || return

  local expiry days_left
  expiry=$(openssl x509 -enddate -noout -in "$cert" | cut -d= -f2)
  days_left=$(( ( $(date -d "$expiry" +%s) - $(date +%s) ) / 86400 ))

  curl -sf -X POST "${API_BASE_URL:-http://localhost:3000}/api/metrics" \
    -H "Content-Type: application/json" \
    -d "{\"source\":\"ssl\",\"type\":\"cert_expiry_days\",\"value\":${days_left},\"domain\":\"${DOMAIN}\"}" \
    || log "WARN: failed to report cert metrics"
}

check_expiry
renew
report_metrics
