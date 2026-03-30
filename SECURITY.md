# 🔒 Security Hardening & Protocols

> **StellarEscrow Production Security Standards**  
> Version: 1.0 | Last Updated: March 2026 | Audience: DevOps, Security, Engineering

---

## Table of Contents

- [1. Network Security](#1-network-security)
- [2. Secrets Management](#2-secrets-management)
- [3. Vulnerability Scanning](#3-vulnerability-scanning)
- [4. Monitoring & Incident Response](#4-monitoring--incident-response)
- [5. Access Control & IAM](#5-access-control--iam)
- [6. Data Protection](#6-data-protection)
- [7. Compliance & Audit](#7-compliance--audit)
- [8. Security Checklist](#8-security-checklist)

---

## 1. Network Security

### 1.1 Kubernetes NetworkPolicies

All network traffic in the StellarEscrow cluster follows **least-privilege** principles. Default-deny ingress and egress policies are applied to all namespaces.

#### Policy Architecture

```
┌─────────────────────────────────────────────────────────┐
│               Ingress (External Traffic)                │
│  nginx-ingress (TLS termination, WAF rules)             │
└──────────────────┬──────────────────────────────────────┘
                   │
     ┌─────────────┴──────────────┐
     │                            │
┌────▼──────┐             ┌──────▼─────┐
│   API      │             │   Client   │
│  (4000)    │             │   (3001)   │
└────┬──────┘             └──────┬─────┘
     │                           │
     └──────────────┬────────────┘
                    │
     ┌──────────────┼──────────────┐
     │              │              │
┌────▼────┐   ┌────▼────┐   ┌────▼────┐
│PostgreSQL│   │  Redis  │   │Indexer  │
│ (5432)   │   │ (6379)  │   │ (3000)  │
└──────────┘   └─────────┘   └─────────┘
```

#### Egress Rules for Sensitive Operations

- **API → PostgreSQL (TCP/5432):** Queries only
- **API → Redis (TCP/6379):** Cache reads/writes
- **Indexer → Stellar RPC (TCP/443):** HTTPS webhooks  
- **Monitoring → All pods (TCP/9090-9100):** Metrics scraping

**Key Principle:** No pod-to-pod communication occurs without explicit NetworkPolicy rules.

### 1.2 Firewall Configuration (AWS/GCP)

**Inbound Rules:**
- TCP/80 → nginx (HTTP redirect to HTTPS)
- TCP/443 → nginx (TLS termination)
- TCP/22 → Bastion host only (SSH)

**Outbound Rules:**
- TCP/443 → Stellar testnet/mainnet RPC endpoints
- TCP/443 → AWS Secrets Manager / HashiCorp Vault
- TCP/443 → Slack webhook API
- TCP/25 → SMTP (optional, for email alerts)

### 1.3 TLS & mTLS

- **Client-to-API:** TLS 1.3, enforced via nginx
- **API-to-PostgreSQL:** Optional mTLS for production (requires cert rotation)
- **Pod-to-Pod:** Istio mTLS recommended for service mesh

---

## 2. Secrets Management

### 2.1 HashiCorp Vault Integration

**Production-Grade Secret Storage** for API keys, database credentials, and private keys.

#### Vault Setup

```bash
# Deploy Vault in HA mode
helm install vault hashicorp/vault \
  --namespace vault \
  --create-namespace \
  --values vault-values.yaml

# Initialize Vault (run once)
kubectl exec -it vault-0 -n vault -- vault operator init \
  -key-shares=5 \
  -key-threshold=3

# Unseal Vault with threshold keys
kubectl exec -it vault-0 -n vault -- vault operator unseal <key-1>
kubectl exec -it vault-0 -n vault -- vault operator unseal <key-2>
kubectl exec -it vault-0 -n vault -- vault operator unseal <key-3>
```

#### Secret Paths & Rotation

| Secret Type | Path | Rotation | Owner |
|-------------|------|----------|-------|
| DB Password | `secret/postgres/prod` | 90 days | DevOps |
| API Keys | `secret/stellar/testnet-key` | 180 days | Platform |
| Signing Keys | `secret/escrow/signing-key` | Never (rotate with contract) | Security |
| AWS Credentials | `aws/creds/s3-access` | 2 hours (dynamic) | IAM |
| Slack Webhook | `secret/alerts/slack-webhook` | 365 days | On-call |

#### Application Integration (Node.js)

```typescript
// Example: Node.js Vault client
import * as VaultClient from '@hashicorp/vault-api';

const client = new VaultClient.ApiClient({
  address: process.env.VAULT_ADDR,
  token: process.env.VAULT_TOKEN,
});

const dbSecret = await client.logical.read('secret/postgres/prod');
const { username, password } = dbSecret.data.data;
```

### 2.2 AWS Secrets Manager Integration

**Alternative for AWS-hosted deployments** (CodeBuild, Lambda, RDS).

```json
{
  "SecretId": "stellar-escrow/prod/db-credentials",
  "SecretString": "{\"username\": \"postgres\", \"password\": \"...\"}",
  "Tags": [
    {"Key": "Environment", "Value": "Production"},
    {"Key": "Rotation", "Value": "90d"}
  ]
}
```

#### Automatic Rotation

```python
# Lambda rotation function
import boto3
import psycopg2

def rotate_db_password(secret_id, client_request_token):
    """Rotate RDS password every 90 days."""
    secrets = boto3.client('secretsmanager')
    rds = boto3.client('rds')
    
    secret = secrets.get_secret_value(SecretId=secret_id)
    old_creds = json.loads(secret['SecretString'])
    
    # Generate new password
    new_password = secrets.generate_secret_value(PasswordLength=32)['SecretString']
    
    # Update RDS master password
    rds.modify_db_instance(
        DBInstanceIdentifier='stellar-escrow-db',
        MasterUserPassword=new_password,
        ApplyImmediately=True
    )
    
    # Store in Secrets Manager
    secrets.put_secret_value(
        SecretId=secret_id,
        ClientRequestToken=client_request_token,
        SecretString=json.dumps({
            **old_creds,
            'password': new_password
        })
    )
```

---

## 3. Vulnerability Scanning

### 3.1 Container Image Scanning (Trivy)

**Automated scanning** of all Docker images—both runtime and build-time.

#### GitHub Actions Workflow

The following workflow runs on every pull request and push to `main`:

- Scans container images for CVEs
- Fails build if HIGH/CRITICAL severity found
- Uploads results to GitHub Security tab

#### Dependency Scanning

- **Node.js:** `npm audit` + Snyk
- **Rust:** `cargo audit` + RustSec
- **Python:** `bandit` + Safety

### 3.2 SBOM & Supply Chain Security

**Software Bill of Materials (SBOM)** generated in CycloneDX format:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<bom xmlns="http://cyclonedx.org/schema/bom/1.3">
  <metadata>
    <component type="application">
      <name>stellar-escrow-api</name>
      <version>1.0.0</version>
    </component>
  </metadata>
  <components>
    <component type="library">
      <name>axios</name>
      <version>1.6.2</version>
      <purl>pkg:npm/axios@1.6.2</purl>
    </component>
  </components>
</bom>
```

---

## 4. Monitoring & Incident Response

### 4.1 Prometheus Alert Rules

**Real-time detection** of unauthorized access, anomalies, and misconfigurations.

#### Alert Thresholds

| Alert | Condition | Severity | Action |
|-------|-----------|----------|--------|
| `UnauthorizedAPIAccess` | >10 failed auth/min per IP | CRITICAL | Block IP, notify security |
| `SuspiciousTradeAmount` | >$100k single transaction | WARNING | Manual review flag |
| `DBConnectionLeak` | >80% max connections | CRITICAL | Scale up, page oncall |
| `HighErrorRate` | API 5xx rate >5% | CRITICAL | Rollback / scale |
| `VaultKeyRotationMissed` | Key age >90 days | WARNING | Trigger immediate rotation |
| `CVEDiscovered` | New HIGH/CRITICAL CVE | CRITICAL | Scan images, create incident |

### 4.2 Grafana Dashboards

**Three-tier monitoring dashboard:**

1. **Overview Dashboard** — System health, request rates, error rates
2. **Security Dashboard** — Failed logins, suspicious activity, alert status
3. **Blockchain Dashboard** — Stellar RPC health, trade volume, dispute rate

### 4.3 Logging & Log Aggregation

All logs stream to **Grafana Loki** with structured fields:

```json
{
  "timestamp": "2026-03-30T14:25:33Z",
  "level": "ERROR",
  "service": "api",
  "message": "Unauthorized access attempt",
  "user_id": "redacted_hash",
  "ip_address": "192.0.2.1",
  "action": "list-trades",
  "status_code": 401
}
```

---

## 5. Access Control & IAM

### 5.1 Kubernetes RBAC

**Role-Based Access Control (RBAC)** for cluster administration:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: api-admin
  namespace: production
rules:
  - apiGroups: ["apps"]
    resources: ["deployments", "statefulsets"]
    verbs: ["get", "list", "patch", "update"]
  - apiGroups: ["v1"]
    resources: ["secrets"]
    verbs: ["get"] # No delete/create
```

### 5.2 AWS IAM Policies

**Least-privilege policies** attached to EC2 roles, ECS tasks, and Lambda functions:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "S3DocumentAccess",
      "Effect": "Allow",
      "Action": ["s3:GetObject", "s3:PutObject"],
      "Resource": "arn:aws:s3:::stellar-escrow-docs/*",
      "Condition": {
        "IpAddress": {
          "aws:SourceIp": ["10.0.0.0/8"]
        }
      }
    },
    {
      "Sid": "VaultAccess",
      "Effect": "Allow",
      "Action": ["secretsmanager:GetSecretValue"],
      "Resource": "arn:aws:secretsmanager:*:*:secret:stellar-escrow/*"
    }
  ]
}
```

### 5.3 Database User Separation

**Three-tier database access:**

| User | Role | Permissions | Rotation |
|------|------|-------------|----------|
| `app_user` | Standard connection | SELECT/INSERT/UPDATE | 90 days |
| `app_readonly` | Analytics queries | SELECT only | Never |
| `admin` | Migrations & schema | Full access | 180 days (service account) |

---

## 6. Data Protection

### 6.1 Encryption

**Data-in-Transit:**
- TLS 1.3 for all external communication
- mTLS for pod-to-pod (Istio service mesh)

**Data-at-Rest:**
- PostgreSQL: Transparent Data Encryption (TDE)
- S3: AES-256 server-side encryption
- Redis: Optional encryption (performance consideration)
- Backups: GPG-encrypted + S3 server-side encryption

### 6.2 Backup & Disaster Recovery

**Daily automated backups:**
```bash
# Backup schedule: 02:00 UTC daily
0 2 * * * pg_dump -h postgres -U postgres stellar_escrow | gzip | \
  gpg --symmetric --cipher-algo AES256 | \
  aws s3 cp - s3://stellar-escrow-backups/db/$(date +%Y-%m-%d).sql.gz.gpg
```

**Retention Policy:**
- Daily: 30 days
- Weekly: 12 weeks
- Monthly: 24 months

### 6.3 PII Handling

Fields that require encryption:
- User email addresses
- Wallet addresses (hashed for lookups)
- Dispute notes
- KYC documents

---

## 7. Compliance & Audit

### 7.1 Audit Logging

**All security events logged to Loki with 365-day retention:**

- Admin console access
- Secret Manager reads
- IAM policy changes
- Contract deployments
- Database schema migrations

### 7.2 Compliance Standards

**StellarEscrow implements controls for:**
- **SOC 2 Type II:** System & organization controls
- **OWASP Top 10:** Web application security
- **NIST Cybersecurity Framework:** Risk management
- **ISO 27001:** Information security management

### 7.3 Incident Response Plan

**Escalation Path:**
1. **Detection** (Automated alerts) → Incident response team notified
2. **Triage** (5 min) → Severity assigned based on impact
3. **Containment** (15 min) → Affected system isolated
4. **Investigation** (1-4 hours) → Root cause analysis
5. **Remediation** (varies) → Patch deployed via change control
6. **Post-Mortem** (within 48 hours) → RCA document created

---

## 8. Security Checklist

### Pre-Deployment

- [ ] All secrets stored in Vault/Secrets Manager (no hardcoded keys)
- [ ] NetworkPolicies applied to all namespaces
- [ ] RBAC roles follow least-privilege principle
- [ ] TLS certificates valid and auto-renewal configured
- [ ] Database encryption enabled (TDE or EBS encryption)
- [ ] Backup strategy tested (full restore validation)
- [ ] Container images scanned for CVEs
- [ ] Dependencies audited (npm audit, cargo audit)

### Post-Deployment

- [ ] Monitoring & alerting verified (test alerts fire correctly)
- [ ] Log aggregation confirmed (Loki receiving all logs)
- [ ] Incident response runbooks updated
- [ ] Access logs reviewed for anomalies
- [ ] Secret rotation scheduled in calendar

### Ongoing (Monthly)

- [ ] Review security logs for suspicious activity
- [ ] Validate backup integrity
- [ ] Update SBOM in artifact registry
- [ ] Conduct security review of new pull requests
- [ ] Rotate non-dynamic secrets

---

## 9. Contact & Escalation

**Security Email:** security@stellar-escrow.dev  
**On-Call Rotation:** [PagerDuty link]  
**CISO:** [Name, contact]  
**Incident Report:** [Internal wiki link]

---

## 10. References & Resources

- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [OWASP Top 10 2021](https://owasp.org/Top10/)
- [CIS Kubernetes Benchmark](https://www.cisecurity.org/benchmark/kubernetes)
- [Stellar Security Best Practices](https://developers.stellar.org/docs/learn/security)
- [Soroban Smart Contract Audits](https://sorobanexamples.com/)
