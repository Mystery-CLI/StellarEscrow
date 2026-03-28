# StellarEscrow — Security Hardening Guide

**Version:** 1.0 | **Owner:** Platform Team | **Last Updated:** 2026-03-27

---

## 1. Network Security Policies

### Kubernetes NetworkPolicy

All pods in the `stellar-escrow` namespace operate under a **default-deny-all** policy (`k8s/base/network-policy.yaml`). Explicit allow rules are defined per service:

| Service      | Accepts traffic from  | Egress allowed to                              |
| ------------ | --------------------- | ---------------------------------------------- |
| `client`     | ingress-nginx         | —                                              |
| `api`        | client, ingress-nginx | postgres:5432, indexer:3000, DNS               |
| `indexer`    | api, prometheus       | postgres:5432, redis:6379, :443 (Horizon), DNS |
| `postgres`   | api, indexer, backup  | —                                              |
| `redis`      | indexer               | —                                              |
| `prometheus` | —                     | indexer:3000, DNS                              |

### Docker Compose (non-Kubernetes)

- Internal services (postgres, redis, prometheus, grafana, loki, alertmanager) bind to `127.0.0.1` only — not exposed on `0.0.0.0`.
- Redis requires password authentication (`REDIS_PASSWORD` env var).
- Containers run with `no-new-privileges:true` and `read_only: true` where applicable.

### Nginx / TLS

- TLS 1.2 and 1.3 only; weak ciphers disabled.
- HSTS with `max-age=63072000; includeSubDomains; preload`.
- Security headers: `CSP`, `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy`, `Permissions-Policy`.
- OCSP stapling enabled.

---

## 2. Secrets Management

### Production (Kubernetes)

Secrets are sourced from **AWS Secrets Manager** via the [Secrets Store CSI Driver](https://secrets-store-csi-driver.sigs.k8s.io/).

Secret paths in AWS Secrets Manager:

```
stellar-escrow/production/database   → DATABASE_URL, POSTGRES_PASSWORD
stellar-escrow/production/api-keys   → API_KEYS, ADMIN_KEYS
stellar-escrow/production/stellar    → CONTRACT_ID
```

The `SecretProviderClass` (`k8s/base/secrets-store.yaml`) syncs these into the `stellar-escrow-secrets` Kubernetes Secret, which is mounted into pods. Pods never receive secrets via environment variables from plaintext manifests.

### Secret Rotation

Run `scripts/secrets-rotate.sh` to trigger rotation in AWS Secrets Manager and restart affected pods:

```bash
# Dry run first
./scripts/secrets-rotate.sh --dry-run

# Execute rotation
AWS_REGION=us-east-1 K8S_NAMESPACE=stellar-escrow ./scripts/secrets-rotate.sh
```

Rotation should be performed:

- Every 90 days (scheduled)
- Immediately after any suspected compromise
- After any team member offboarding

### Local / Docker Compose

- Copy `.env.example` to `.env` and fill in real values.
- Never commit `.env` files — they are gitignored.
- Required secrets: `POSTGRES_PASSWORD`, `REDIS_PASSWORD`, `API_KEYS`, `ADMIN_KEYS`, `GRAFANA_ADMIN_PASSWORD`.

---

## 3. Vulnerability Scanning

The `security-scan` CI workflow (`.github/workflows/security-scan.yml`) runs on every push, PR, and nightly:

| Scanner       | Target                                      | Threshold                         |
| ------------- | ------------------------------------------- | --------------------------------- |
| `npm audit`   | Node.js deps (api, client, components, app) | Fail on `critical`                |
| `cargo audit` | Rust deps (contract, indexer)               | Fail on any advisory              |
| CodeQL        | TypeScript/JavaScript source                | `security-extended` queries       |
| Trivy         | Docker images (api, client)                 | Fail on `CRITICAL`/`HIGH` unfixed |
| Gitleaks      | Full git history                            | Fail on any secret leak           |
| Checkov       | Terraform IaC                               | Fail on Terraform; warn on K8s    |

Results are uploaded to the **GitHub Security tab** (SARIF format) for triage.

### Running locally

```bash
# Node.js audit
cd api && npm audit --audit-level=high

# Rust audit
cargo audit

# Container scan (requires Docker + Trivy)
trivy image --severity CRITICAL,HIGH stellar-escrow-api:latest

# Secret scan (requires gitleaks)
gitleaks detect --source . --verbose

# Full local security audit
./scripts/security-audit.sh
```

---

## 4. Security Monitoring

Security alerts are defined in `monitoring/alert_rules_security.yml` and routed through Alertmanager.

### Alert Categories

**Authentication & Access**

- `HighAuthFailureRate` — >10 auth failures/sec for 2m → possible brute-force
- `AdminKeyUsedFromUnknownIP` — admin key from untrusted IP → immediate critical alert

**Rate Limiting / DDoS**

- `RateLimitThrottlingSpike` — >50 throttled requests/sec → possible DDoS

**TLS / Certificates**

- `TLSCertExpiryWarning` — cert expires in <30 days
- `TLSCertExpiryCritical` — cert expires in <7 days

**Anomalous Traffic**

- `UnexpectedHTTPMethods` — TRACE/CONNECT/OPTIONS detected
- `SuspiciousPayloadSize` — request body >10 MB

**Database**

- `DatabaseConnectionExhaustion` — >90% connection pool used

**Container / Host**

- `ContainerRunningAsRoot` — root process in container
- `UnexpectedPrivilegedContainer` — privileged container in namespace

### Runbooks

| Alert                       | Runbook                                                       |
| --------------------------- | ------------------------------------------------------------- |
| `HighAuthFailureRate`       | Block source IP at nginx/WAF; review API key exposure         |
| `AdminKeyUsedFromUnknownIP` | Rotate admin keys immediately via `scripts/secrets-rotate.sh` |
| `TLSCertExpiryCritical`     | Run `scripts/renew-certs.sh` manually; check certbot logs     |
| `RateLimitThrottlingSpike`  | Review nginx access logs; consider IP-level block             |
| `ContainerRunningAsRoot`    | Audit Dockerfile USER directive; redeploy                     |

---

## 5. Pod Security & RBAC

- Namespace enforces **Pod Security Standard: restricted** (no privileged containers, no host namespaces, read-only root filesystem required).
- Each service has a dedicated `ServiceAccount` with `automountServiceAccountToken: false`.
- A `Role` grants read-only access to `stellar-escrow-secrets` only — no wildcard permissions.

---

## 6. Compliance & Audit Trail

- All admin API calls are logged with actor, timestamp, and source IP.
- Prometheus retains metrics for 30 days; Loki retains logs per `monitoring/loki.yml`.
- AML/KYC compliance alerts are defined in `monitoring/alert_rules.yml`.
- Backup integrity is verified on every restore via `scripts/dr-test.sh`.

---

## 7. Security Checklist (Pre-Deployment)

Run before every production deployment:

```bash
./scripts/security-audit.sh
```

Manual checklist:

- [ ] All secrets sourced from AWS Secrets Manager (not hardcoded)
- [ ] `npm audit` and `cargo audit` pass with no critical findings
- [ ] Trivy scan passes for all images being deployed
- [ ] Network policies applied and verified with `kubectl get networkpolicy -n stellar-escrow`
- [ ] TLS certificate valid for >30 days
- [ ] Grafana admin password is non-default
- [ ] Redis password set
- [ ] Monitoring alerts firing correctly (test with `amtool alert add` in alertmanager)
