# StellarEscrow — Cloud Cost Analysis

## Current Infrastructure Inventory

| Service | Resource | Est. Monthly Cost |
|---------|----------|-------------------|
| ECS Fargate (indexer) | 0.5 vCPU / 1 GB, always-on | ~$15 |
| ECS Fargate (api) | 0.5 vCPU / 1 GB, always-on | ~$15 |
| RDS PostgreSQL | db.t3.micro, 20 GB **gp3** | ~$24 |
| ALB | 1 load balancer + LCU | ~$20 |
| NAT Gateway | 1 AZ, ~10 GB/month | ~$35 |
| ECR | 2 repos, ~500 MB images | ~$1 |
| S3 (backups) | ~5 GB STANDARD_IA → Glacier IR after 90d | ~$1 |
| CloudWatch Logs | ~2 GB/month, 30-day retention | ~$2 |
| Secrets Manager | 2 secrets | ~$1 |
| **Total (estimated)** | | **~$114/month** |

## Cost Breakdown by Category

```
Compute (ECS Fargate):  ~26%   $30
Database (RDS):         ~21%   $24
Networking (NAT/ALB):   ~48%   $55
Storage/Other:           ~5%    $5
```

## Cost Monitoring

| Tool | Purpose |
|------|---------|
| AWS Budgets (`cost-monitoring.tf`) | Email alerts at 80% / 100% / 110% forecast |
| CloudWatch anomaly detection | Billing spike detection |
| Prometheus + Grafana (`cost.json`) | Real-time cost dashboard |
| `alert_rules_cost.yml` | Budget warning/exceeded + cost spike + low utilisation alerts |
| `cost-report.sh` | CLI monthly report with MoM comparison |
| GitHub Actions (`cost-report.yml`) | Automated monthly report on 1st of each month |

## Implemented Optimizations

| Optimization | File | Saving |
|-------------|------|--------|
| RDS gp2 → gp3 storage | `cost-optimization.tf` | ~$0.70/month per 20 GB |
| CloudWatch log retention (30d prod / 14d staging) | `resource-optimization.tf` | ~$1-2/month at scale |
| ECR lifecycle: keep last 5 images | `resource-optimization.tf` | Prevents cost creep |
| S3 STANDARD_IA → Glacier IR after 90d | `cost-optimization.tf` | ~20-40% on backup storage |
| S3 abort incomplete multipart uploads | `cost-optimization.tf` | Eliminates hidden charges |
| Cost allocation tags + resource group | `cost-optimization.tf` | Enables per-service breakdown |
| Docker resource limits (indexer/api) | `docker-compose.yml` | Prevents resource over-provisioning |

## Identified Optimization Opportunities

### High Impact

1. **NAT Gateway → NAT Instance**
   - Current: NAT Gateway ~$35/month
   - Optimized: t3.nano NAT instance ~$4/month
   - Saving: ~$31/month
   - Risk: Single point of failure (acceptable for non-HA dev/staging)

2. **RDS gp2 → gp3 storage**
   - gp2 20 GB: ~$2.30/month
   - gp3 20 GB: ~$1.60/month + free 3000 IOPS baseline
   - Saving: ~$0.70/month + better performance

3. **ECS Fargate Spot for non-critical tasks**
   - Spot pricing: ~70% discount on Fargate
   - Apply to: indexer background tasks, batch analytics
   - Saving: ~$10/month

### Medium Impact

4. **CloudWatch Logs retention**
   - Set retention to 30 days (currently unlimited)
   - Saving: ~$1-2/month at scale

5. **ECR lifecycle policies**
   - Keep only last 5 images per repo
   - Saving: minimal now, prevents cost creep

6. **S3 Intelligent-Tiering for backups**
   - Auto-moves infrequently accessed objects to cheaper tiers
   - Saving: ~20-40% on backup storage at scale

### Low Impact / Future

7. **Reserved Instances for RDS** (if production traffic is stable)
   - 1-year reserved: ~40% discount
   - Saving: ~$10/month

8. **Savings Plans for Fargate** (if usage is predictable)
   - 1-year compute savings plan: ~20% discount
   - Saving: ~$6/month

## Recommended Actions by Environment

| Environment | Action | Priority |
|-------------|--------|----------|
| Development | Use NAT instance, scale-to-zero ECS | High |
| Staging | gp3 storage, log retention 14 days | Medium |
| Production | gp3 storage, Fargate Spot for batch, Reserved RDS | High |
