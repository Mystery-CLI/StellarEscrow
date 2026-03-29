#!/usr/bin/env bash
# scripts/security-audit.sh
# Local security audit: checks for hardcoded secrets, insecure configs, and
# validates Docker/K8s security posture before deployment.
#
# Usage: ./scripts/security-audit.sh

set -euo pipefail

PASS=0
FAIL=0
WARN=0

log()  { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }
pass() { echo "  ✅ $*"; ((PASS++)); }
fail() { echo "  ❌ $*"; ((FAIL++)); }
warn() { echo "  ⚠️  $*"; ((WARN++)); }

# ── 1. Check for hardcoded secrets in tracked files ──────────────────────────
log "Checking for hardcoded secrets..."
SECRET_PATTERNS=(
  "password\s*=\s*['\"][^'\"]{4,}"
  "api_key\s*=\s*['\"][^'\"]{8,}"
  "secret\s*=\s*['\"][^'\"]{8,}"
  "POSTGRES_PASSWORD.*=.*password"
)
FOUND_SECRETS=false
for pattern in "${SECRET_PATTERNS[@]}"; do
  if git grep -rniE "$pattern" -- '*.toml' '*.yml' '*.yaml' '*.env*' 2>/dev/null | \
     grep -v '.env.example' | grep -v '#' | grep -q .; then
    fail "Possible hardcoded secret matching: ${pattern}"
    FOUND_SECRETS=true
  fi
done
[[ "$FOUND_SECRETS" == "false" ]] && pass "No obvious hardcoded secrets found"

# ── 2. Verify .env files are gitignored ──────────────────────────────────────
log "Checking .gitignore coverage..."
for env_file in .env .env.local .env.production.local; do
  if git check-ignore -q "$env_file" 2>/dev/null; then
    pass "${env_file} is gitignored"
  else
    warn "${env_file} is NOT gitignored — verify it doesn't contain real secrets"
  fi
done

# ── 3. Docker: non-root user check ───────────────────────────────────────────
log "Checking Dockerfiles for non-root USER..."
for df in $(find . -name 'Dockerfile' -not -path '*/.git/*'); do
  if grep -q '^USER ' "$df"; then
    pass "${df}: non-root USER set"
  else
    fail "${df}: no USER directive — container may run as root"
  fi
done

# ── 4. K8s secrets: no plaintext passwords ───────────────────────────────────
log "Checking Kubernetes secrets for plaintext passwords..."
if grep -rn 'password:' k8s/ 2>/dev/null | grep -v '#' | grep -qv 'secretsmanager'; then
  warn "k8s/base/secrets.yaml contains plaintext values — migrate to Secrets Store CSI"
else
  pass "No plaintext passwords in k8s manifests"
fi

# ── 5. TLS config present ─────────────────────────────────────────────────────
log "Checking TLS configuration..."
if [[ -f ssl.nginx.conf ]] && grep -q 'TLSv1.2' ssl.nginx.conf; then
  pass "Nginx TLS config present with TLS 1.2/1.3"
else
  fail "ssl.nginx.conf missing or TLS not configured"
fi

# ── 6. Security headers in nginx ─────────────────────────────────────────────
log "Checking security headers..."
for header in "Strict-Transport-Security" "Content-Security-Policy" "X-Frame-Options" "X-Content-Type-Options"; do
  if grep -q "$header" ssl.nginx.conf 2>/dev/null; then
    pass "Header present: ${header}"
  else
    fail "Missing security header: ${header}"
  fi
done

# ── 7. Network policies exist ────────────────────────────────────────────────
log "Checking network policies..."
if [[ -f k8s/base/network-policy.yaml ]]; then
  POLICY_COUNT=$(grep -c 'kind: NetworkPolicy' k8s/base/network-policy.yaml || true)
  pass "Network policies defined (${POLICY_COUNT} policies)"
else
  fail "k8s/base/network-policy.yaml not found"
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo "────────────────────────────────────────"
echo "Security Audit Summary"
echo "  ✅ Passed:   ${PASS}"
echo "  ⚠️  Warnings: ${WARN}"
echo "  ❌ Failed:   ${FAIL}"
echo "────────────────────────────────────────"

if [[ $FAIL -gt 0 ]]; then
  log "Audit FAILED — ${FAIL} issue(s) require attention."
  exit 1
else
  log "Audit PASSED."
  exit 0
fi
