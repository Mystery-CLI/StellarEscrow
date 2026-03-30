# 🔐 Implementation Guide: Security Hardening, File Storage, Fraud Detection & Stellar Indexing

> **StellarEscrow Production Features** | March 2026  
> Overview of security hardening, file storage, fraud detection, and event indexing systems.

---

## 📋 Table of Contents

- [1. Security Hardening](#1-security-hardening)
- [2. File Storage Service](#2-file-storage-service)
- [3. Fraud Detection System](#3-fraud-detection-system)
- [4. Stellar Event Indexer](#4-stellar-event-indexer)
- [5. Architecture Diagram](#5-architecture-diagram)
- [6. Deployment Instructions](#6-deployment-instructions)
- [7. Monitoring & Alerts](#7-monitoring--alerts)

---

## 1. Security Hardening

### Components Implemented

#### ✅ Kubernetes NetworkPolicies (`/k8s/network-policies.yaml`)
- **Least-privilege traffic isolation** for all pods
- **Default-deny** ingress and egress rules
- **Per-service policies** for API, Indexer, PostgreSQL, Redis
- **Monitoring namespace** separate for Prometheus/Grafana

**Key Features:**
- API can reach: PostgreSQL (5432), Redis (6379), Stellar RPC (443)
- PostgreSQL accepts connections from: API, Indexer, Backup service
- All external communication via nginx ingress controller only

**Deployment:**
```bash
# Apply Kubernetes network policies
kubectl apply -f k8s/network-policies.yaml

# Verify policies
kubectl get networkpolicies -n production
kubectl describe networkpolicy allow-api -n production
```

---

#### ✅ Secrets Management (`/infra/vault-values.yaml`)

**HashiCorp Vault Integration:**
- **HA Setup** with 3 replicas and Raft storage
- **Auto-unsealing** support (AWS KMS)
- **Kubernetes Auth** for service-to-service authentication
- **Secret rotation** via CronJob (weekly)

**Supported Secrets:**
```
secret/postgres/prod          → Database credentials (90-day rotation)
secret/stellar/testnet-key    → Testnet private key
secret/stellar/mainnet-key    → Mainnet private key (immutable after deployment)
secret/alerts/slack-webhook   → Slack notification URL
aws/creds/s3-access          → Dynamic AWS credentials (2-hour TTL)
```

**Deployment:**
```bash
# Install Vault via Helm
helm install vault hashicorp/vault -f infra/vault-values.yaml -n vault --create-namespace

# Initialize Vault (one time)
kubectl exec -it vault-0 -n vault -- vault operator init -key-shares=5 -key-threshold=3

# Enable Kubernetes auth
kubectl exec vault-0 -n vault -- vault auth enable kubernetes

# Create service account policies
kubectl exec vault-0 -n vault -- vault write auth/kubernetes/role/api \
  bound_service_account_names=api \
  bound_service_account_namespaces=production \
  policies=api-policy \
  ttl=24h
```

---

#### ✅ GitHub Actions Vulnerability Scanning (`.github/workflows/security.yml`)

**Integrated Tools:**
1. **Trivy** — Container image & filesystem scanning
2. **npm audit** — JavaScript dependency auditing
3. **cargo audit** — Rust dependency scanning
4. **Snyk** — Supply chain security & license compliance
5. **CodeQL** — Static code analysis (SAST)
6. **Secrets Detection** — Prevent credential commits

**Triggers:**
- On every push to `main` and `develop`
- On pull requests
- Weekly scheduled scan (Sundays 00:00 UTC)

**Example Output:**
```bash
# Run locally
npm run test:security

# GitHub Actions will:
# 1. Build container images
# 2. Scan for CVEs (HIGH/CRITICAL fail build)
# 3. Audit dependencies
# 4. Generate SBOM (Software Bill of Materials)
# 5. Upload results to Security tab
```

---

#### ✅ Prometheus/Grafana Monitoring (`/monitoring/prometheus-rules.yaml`)

**Alert Categories:**
1. **Authentication** — Unauthorized access attempts, token rotation
2. **Fraud** — High-value transactions, anomalies
3. **Infrastructure** — Container vulnerabilities, certificate expiration
4. **Blockchain** — RPC latency, event backlog
5. **Data Protection** — Backup failures, replication lag
6. **Compliance** — Audit log gaps, unauthorized admin access

**Example Alerts:**
```yaml
# Unauthorized API access (>10 failed auth/min)
UnauthorizedAPIAccess:
  condition: increase(http_requests_total{status="401"}[5m]) > 10
  severity: critical
  action: Block IP, notify security team

# High fraud risk score
FraudScoreAnomalyDetected:
  condition: fraud_risk_score > 0.85
  severity: critical
  action: Manual review + Slack notification
```

**Dashboard Access:**
```
URL: https://<your-domain>/grafana
Default credentials: admin/admin (CHANGE IMMEDIATELY)
Dashboards:
  - Overview: System health, request rates
  - Security: Failed logins, suspicious activity
  - Blockchain: RPC health, trade volume
```

---

#### ✅ SECURITY.md Documentation

Comprehensive security protocol covering:
- Network security & firewalls
- Secrets management & rotation
- Vulnerability scanning pipeline
- Monitoring & incident response
- RBAC & IAM policies
- Data protection & encryption
- Compliance requirements (SOC 2, OWASP, NIST, ISO 27001)

---

## 2. File Storage Service

### Architecture

```
Upload Flow:
User → Express API → Multer (in-memory) → Sharp (compression) → S3

Download Flow:
User → Express API → Vault (signed URL) → S3 (direct download)
```

### REST APIs

#### ✅ POST `/storage/upload`
**Upload avatar or legal document**

```typescript
// Request
POST /api/storage/upload
Content-Type: multipart/form-data

{
  "file": <binary>,
  "fileType": "avatar" | "document",
  "userId": "user_123"
}

// Response (201 Created)
{
  "success": true,
  "fileId": "file_1a2b3c4d5e6f7g8h...",
  "fileName": "avatar.webp",
  "size": 184320,
  "compressed": true,
  "uploadedAt": "2026-03-30T14:25:33Z"
}
```

**File Type Validation:**
- **Avatars**: PNG, JPEG, WebP (max 5MB)
- **Documents**: PDF only (max 10MB)

**Auto-Compression:**
- Images compressed to WebP (80% quality)
- Resized to 400x400 (square avatar)
- ~70% size reduction typical

---

#### ✅ GET `/storage/download/:fileId`
**Generate time-limited signed URL**

```typescript
// Request
GET /api/storage/download/file_1a2b3c4d5e6f7g8h...
Header: x-user-id: user_123

// Response
{
  "fileId": "file_1a2b3c4d5e6f7g8h...",
  "url": "https://s3.amazonaws.com/...?X-Amz-Signature=...",
  "expiresIn": 86400,
  "expiresAt": "2026-03-31T14:25:33Z"
}
```

**Security Features:**
- Signed URL valid for 24 hours only
- AES-256 encryption in transit
- Owner-only access verification
- S3 versioning enabled for recovery

---

#### ✅ GET `/storage/files`
**List user's files with metadata**

```typescript
// Request
GET /api/storage/files
Header: x-user-id: user_123

// Response
{
  "success": true,
  "count": 2,
  "files": [
    {
      "fileId": "file_abc123",
      "originalName": "avatar.png",
      "size": 184320,
      "mimeType": "image/webp",
      "uploadedAt": "2026-03-30T10:00:00Z",
      "expiresAt": null
    },
    {
      "fileId": "file_def456", 
      "originalName": "legal_agreement.pdf",
      "size": 1048576,
      "mimeType": "application/pdf",
      "uploadedAt": "2026-03-28T08:30:00Z",
      "expiresAt": "2026-04-04T08:30:00Z"
    }
  ]
}
```

---

#### ✅ DELETE `/storage/files/:fileId`
**Permanently delete file (owner only)**

```typescript
// Request
DELETE /api/storage/files/file_abc123
Header: x-user-id: user_123

// Response
{
  "success": true,
  "message": "File deleted successfully"
}
```

---

### S3 Configuration

**Bucket Security:**
- Versioning enabled (recover deleted files)
- Server-side encryption (AES-256)
- Public access blocked
- Lifecycle policy: Archive after 90 days

**IAM Policy:**
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": ["s3:GetObject", "s3:PutObject"],
      "Resource": "arn:aws:s3:::stellar-escrow-storage/*",
      "Condition": {
        "IpAddress": {"aws:SourceIp": ["10.0.0.0/8"]}
      }
    }
  ]
}
```

---

## 3. Fraud Detection System

### Rule-Based Detection

**Rule Scoring (0.0 - 1.0):**

| Rule | Condition | Score |
|------|-----------|-------|
| Large Transaction | Amount > $10,000 | +0.30 |
| New Account | Age < 7 days | +0.25 |
| High Velocity | >5 trades/minute | +0.35 |
| Unusual Time | 3-5am UTC | +0.10 |
| Duplicate Pattern | Same amount 3× in 1h | +0.20 |

**Example:**
- New user, $15k trade, 6 trades/min → **0.90 score** (HIGH RISK)
- Established user, $500 trade, normal velocity → **0.15 score** (LOW RISK)

---

### ML-Based Detection

**Python Service (`/api/src/services/ml_fraud_service.py`)**

**Ensemble Model:**
- Random Forest (60% weight)
- Gradient Boosting (40% weight)
- Feature scaling with StandardScaler

**Features:**
```python
FEATURE_NAMES = [
  'purchase_amount',           # Transaction size
  'user_age_days',             # Account age
  'previous_trades',           # Historical activity
  'disputes_ratio',            # Past disputes %
  'time_since_last_trade_hours',
  'trades_last_24h',
  'trades_last_hour',
  'avg_transaction_value',
  'location_changes_24h',
  'device_fingerprint_change',  # Boolean
]
```

**Prediction Endpoint:**
```python
# POST /predict
{
  "features": {
    "purchase_amount": 15000.50,
    "user_age_days": 3,
    ...
  }
}

# Response
{
  "fraud_probability": 0.85,     # 0-1 scale
  "risk_level": "critical",      # low, medium, high, critical
  "confidence": 0.95,            # |rf_prob - gb_prob|
  "rf_probability": 0.87,
  "gb_probability": 0.82
}
```

---

### Fraud Alerts & Slack Integration

When `combined_score > 0.8`, the system:
1. **Logs to Slack:**
   ```
   🚨 CRITICAL FRAUD ALERT
   Trade: 0x1234...
   Seller: alice@example.com
   Amount: $15,000 USDC
   Risk Score: 85%
   Rules Triggered: large_transaction_value, abnormal_transaction_velocity
   [Review in Dashboard] [Approve] [Block]
   ```

2. **Updates trade status to `PENDING_REVIEW`**

3. **Queues for manual arbitration**

4. **Logs to PostgreSQL** for audit trail

---

### Fraud Dashboard Schema

**Tables:**
- `fraud_events` — Detailed fraud analysis records
- `fraud_daily_metrics` — Daily aggregate statistics
- `user_fraud_history` — User risk profiles
- `ml_model_performance` — Model evaluation metrics

**REST Endpoints:**
```
GET  /fraud/stats                  — Daily overview
GET  /fraud/time-series?days=30   — Historical metrics
GET  /fraud/pending-reviews       — Manual review queue
POST /fraud/alerts/:id/review     — Submit review decision
GET  /fraud/user-risk/:userId     — User risk profile
GET  /fraud/model-performance     — ML model metrics
```

---

## 4. Stellar Event Indexer

### Real-Time Event Monitoring

**Workflow:**
```
Stellar Network
      ↓
  RPC getEvents (every 4 sec)
      ↓
  XDR Parsing
      ↓
  PostgreSQL Storage
      ↓
  WebSocket Broadcast
      ↓
  Frontend (Real-time updates)
```

### Supported Event Types

| Event | Data |
|-------|------|
| `trade_created` | Trade ID, seller, amount |
| `trade_funded` | Trade ID, buyer, funded amount |
| `trade_completed` | Trade ID, recipient, amount |
| `dispute_raised` | Trade ID, initiator, reason |
| `dispute_resolved` | Trade ID, winner, arbitrator decision |

### XDR Parsing Example

```typescript
// Raw XDR (base64)
const xdrEvent = "AAAACFhEUEZUVkJMSVVCREFUQQAAAAA...";

// Parsed JSON
{
  "eventType": "trade_created",
  "tradeId": "0x1234abcd...",
  "sellerId": "GBLZ3DCUPXMHSXUKCJWH7GVEJVYUZGZTIVMCVKQ4SLXQWZ3GXYUZEJDA",
  "amount": "10000.00",
  "asset": "USDC",
  "timestamp": "2026-03-30T14:25:33Z"
}
```

---

### WebSocket Streaming

**Client Subscribe:**
```javascript
// Connect
const ws = new WebSocket('wss://api.stellar-escrow.dev/ws/events');

// Subscribe to contract
ws.send(JSON.stringify({
  type: 'subscribe',
  contractIds: ['CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4']
}));

// Receive events
ws.addEventListener('message', (e) => {
  const msg = JSON.parse(e.data);
  if (msg.type === 'event') {
    console.log('New event:', msg.data);
    // Update UI
  }
});
```

**Real-time Updates:**
```json
{
  "type": "event",
  "data": {
    "id": "1234-567811",
    "type": "trade_funded",
    "contractId": "CAAAA...",
    "ledgerSeq": 1234567,
    "ledgerClosedAt": "2026-03-30T14:25:33Z",
    "data": {
      "tradeId": "0xabc123",
      "buyerId": "GBBB...",
      "fundedAmount": "10000.00"
    }
  },
  "timestamp": "2026-03-30T14:25:33Z"
}
```

---

### Historical Replay

**Fetch events from specific ledger range:**
```bash
# Start replay
curl -X POST https://api.stellar-escrow.dev/api/events/replay \
  -H "Content-Type: application/json" \
  -d '{
    "startLedger": 1000000,
    "endLedger": 1100000
  }'

# Response
{
  "success": true,
  "replayId": "replay_1711812333",
  "message": "Replay started from ledger 1000000 to 1100000"
}

# Check status
curl https://api.stellar-escrow.dev/api/events/replay/replay_1711812333
```

---

### Event Query API

**GET `/api/events`** — Query with filters
```
?contractId=CAAAA...&type=trade_created&limit=50&offset=0
```

**GET `/api/contracts/:contractId/stats`** — Contract statistics
```json
{
  "totalEvents": 45230,
  "uniqueEventTypes": 5,
  "firstLedger": 1000000,
  "lastLedger": 1234567,
  "eventTypeBreakdown": [
    {"type": "trade_created", "count": 20000},
    {"type": "trade_funded", "count": 15000},
    ...
  ]
}
```

**GET `/api/events/trade/:tradeId`** — All events for trade
```
Returns: created → funded → completed/disputed → resolved
```

---

## 5. Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    SECURITY HARDENING LAYER                      │
├─────────────────────────────────────────────────────────────────┤
│ ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐│
│ │ Kubernetes       │  │ HashiCorp Vault  │  │ GitHub Actions   ││
│ │ NetworkPolicies  │  │ Secrets Mgmt     │  │ Vuln. Scanning   ││
│ └──────────────────┘  └──────────────────┘  └──────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│                       API GATEWAY LAYER                          │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌──────────────┐  ┌──────────────────────┐│
│ │ File Storage    │  │ Fraud        │  │ Stellar Events API   ││
│ │ (S3 + Signed    │  │ Detection    │  │ (Query + Replay)     ││
│ │  URLs)          │  │ (Rule + ML)  │  │                      ││
│ └─────────────────┘  └──────────────┘  └──────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│                    DATA & STREAMING LAYER                        │
├─────────────────────────────────────────────────────────────────┤
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│ │ PostgreSQL   │  │ Redis Cache  │  │ Stellar Indexer      │   │
│ │ (9,000+ TPS) │  │              │  │ WebSocket Server     │   │
│ └──────────────┘  └──────────────┘  └──────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│                    MONITORING & OBSERVABILITY                    │
├─────────────────────────────────────────────────────────────────┤
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│ │ Prometheus   │  │ Grafana      │  │ Slack Alerts         │   │
│ │ Metrics      │  │ Dashboards   │  │ PagerDuty Escalation │   │
│ └──────────────┘  └──────────────┘  └──────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. Deployment Instructions

### Prerequisites

```bash
# Required tools
- Docker & Docker Compose
- Kubernetes cluster (v1.24+)
- Helm 3.x
- AWS CLI v2 (for S3)
- PostgreSQL 15+
- Node.js 20+
```

### Step 1: Deploy Infrastructure

```bash
# 1. Create Kubernetes namespaces
kubectl create namespace production
kubectl create namespace monitoring
kubectl create namespace vault

# 2. Label namespaces for network policies
kubectl label namespace production kube-cluster=primary
kubectl label namespace monitoring kube-cluster=primary

# 3. Apply network policies
kubectl apply -f k8s/network-policies.yaml

# 4. Deploy Vault
helm install vault hashicorp/vault -f infra/vault-values.yaml -n vault

# 5. Deploy PostgreSQL & Redis
kubectl apply -f k8s/postgres/
kubectl apply -f k8s/redis/
```

### Step 2: Configure Secrets

```bash
# Initialize databases with schema
python scripts/init-databases.py

# Set up Vault secrets
export VAULT_ADDR=https://vault.vault.svc.cluster.local:8200
vault login -method=kubernetes role=admin

# Store secrets
vault kv put secret/postgres/prod username=app_user password="<long-random-pass>"
vault kv put secret/stellar/mainnet-key signing_key="<contract-key>"
vault kv put secret/alerts/slack-webhook url="https://hooks.slack.com/..."
```

### Step 3: Deploy Services

```bash
# Build images
docker build -t stellar-escrow/api:latest api/
docker build -t stellar-escrow/indexer:latest indexer/
docker build -t stellar-escrow/client:latest client/

# Deploy to Kubernetes
kubectl apply -f k8s/api/
kubectl apply -f k8s/indexer/
kubectl apply -f k8s/client/

# Verify
kubectl get pods -n production
kubectl logs -f deployment/api -n production
```

### Step 4: Deploy Monitoring

```bash
# Install Prometheus & Grafana
helm install prometheus prometheus-community/kube-prometheus-stack -f monitoring/values.yaml
helm install loki grafana/loki-stack

# Import dashboards
curl -X POST http://grafana:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @monitoring/dashboards/security-dashboard.json
```

---

## 7. Monitoring & Alerts

### Key Metrics

```
API Performance:
  - p95 latency: < 200ms
  - Error rate: < 0.1%
  - Requests/sec: 1,000+

Fraud Detection:
  - True positive rate: > 95%
  - False positive rate: < 5%
  - Avg review time: < 5 min

Stellar Indexing:
  - Event lag: < 4 seconds
  - Parse success rate: > 99%
  - Database throughput: 1,000 events/sec
```

### Alert Routing

```
Severity    Channel           Escalation
─────────────────────────────────────────────
Critical    Slack + PagerDuty 1min → On-call
High        Slack             5min → Team lead
Medium      Slack             1hr  → Email
Low         Email/Dashboard   Manual review
```

### Health Checks

```bash
# API
curl -s https://api.stellar-escrow.dev/health | jq .

# Indexer
curl -s https://api.stellar-escrow.dev/api/indexer/status | jq .

# Database
psql -h postgres.production.svc -U app_user -d stellar_escrow -c "SELECT version();"

# Vault
vault status

# Prometheus
curl -s http://prometheus:9090/api/v1/query?query=up | jq .
```

---

## 📞 Next Steps

1. **Review** SECURITY.md for compliance requirements
2. **Test** all components in staging before production
3. **Configure** Slack webhooks and alerting
4. **Train** ops team on incident response procedures
5. **Schedule** monthly security reviews

---

**Generated**: 2026-03-30  
**Status**: ✅ Complete
