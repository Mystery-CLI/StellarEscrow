# StellarEscrow Infrastructure

Terraform-managed infrastructure for the StellarEscrow platform.

## Structure

```
infra/
├── bootstrap/          # One-time S3 + DynamoDB state backend setup
│   └── main.tf
├── terraform/          # Main infrastructure
│   ├── main.tf         # Provider + backend config
│   ├── variables.tf    # All input variables
│   ├── locals.tf       # Per-environment derived config
│   ├── modules.tf      # Module wiring + Secrets Manager
│   ├── outputs.tf      # Stack outputs
│   ├── environments/   # Per-environment tfvars
│   │   ├── development.tfvars
│   │   ├── staging.tfvars
│   │   └── production.tfvars
│   └── modules/
│       ├── networking/ # VPC, subnets, NAT gateway
│       ├── database/   # RDS PostgreSQL
│       └── api/        # ECS Fargate, ALB, ECR
└── tests/              # Terratest infrastructure tests
    ├── go.mod
    └── networking_test.go
```

## Infrastructure Versioning

The `locals.infra_version` field in `locals.tf` tracks the infrastructure
schema version independently of the application version. Bump it whenever
the module structure or resource layout changes in a breaking way.

State is versioned automatically via S3 bucket versioning — every `apply`
creates a new state version, enabling point-in-time recovery.

## Prerequisites

- Terraform >= 1.7.0
- AWS CLI configured with appropriate credentials
- Go >= 1.22 (for infrastructure tests)

## First-time Setup (State Backend)

Run once per AWS account to create the S3 bucket and DynamoDB lock table:

```bash
cd infra/bootstrap
terraform init
terraform apply
```

## Usage

### Plan

```bash
cd infra/terraform
terraform init
terraform plan -var-file="environments/staging.tfvars" \
               -var="db_password=$DB_PASSWORD"
```

### Apply

```bash
terraform apply -var-file="environments/production.tfvars" \
                -var="db_password=$DB_PASSWORD" \
                -var="stellar_contract_id=$CONTRACT_ID"
```

### Destroy (non-production only)

```bash
terraform destroy -var-file="environments/development.tfvars" \
                  -var="db_password=$DB_PASSWORD"
```

## Secrets

Secrets are **never** committed. Inject them at apply time:

| Variable | How to inject |
|---|---|
| `db_password` | `TF_VAR_db_password` env var or `-var` flag |
| `stellar_contract_id` | `TF_VAR_stellar_contract_id` or CI secret |
| AWS credentials | `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` |

The DB connection string is stored in AWS Secrets Manager and injected into
ECS tasks at runtime — it never appears in environment variables directly.

## Infrastructure Tests

Plan-only tests run in CI on every PR. Full apply tests require real AWS
credentials and are run manually or in a dedicated test account.

```bash
cd infra/tests
go test -v -timeout 10m -run "Plan" ./...   # plan-only (CI-safe)
go test -v -timeout 30m ./...               # full apply (needs AWS)
```

## CI/CD

The `terraform.yml` workflow:

1. Validates and lints on every PR touching `infra/**`
2. Posts a plan comment per environment on PRs
3. Applies to **staging** on merge to `develop`
4. Applies to **production** on merge to `main` (requires GitHub environment approval)

Required GitHub secrets: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`,
`TF_DB_PASSWORD`, `STELLAR_CONTRACT_ID`.
