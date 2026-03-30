# Cost Optimization — gp3 storage, Fargate Spot, RDS storage type upgrade
# Complements resource-optimization.tf with compute and storage cost reductions.

# ---------------------------------------------------------------------------
# Variables (shared with other cost files)
# ---------------------------------------------------------------------------

variable "monthly_budget_usd" {
  description = "Monthly budget threshold in USD"
  type        = number
  default     = 150
}

variable "alert_email" {
  description = "Email for cost alerts"
  type        = string
  default     = "ops@stellarescrow.io"
}

# ---------------------------------------------------------------------------
# RDS gp3 storage upgrade (20% cheaper than gp2, free 3000 IOPS baseline)
# Apply to existing RDS instances via aws CLI or Terraform import
# ---------------------------------------------------------------------------

# NOTE: To migrate an existing gp2 instance to gp3 without downtime:
#   aws rds modify-db-instance \
#     --db-instance-identifier stellarescrow-production \
#     --storage-type gp3 \
#     --apply-immediately
#
# Terraform resource reference (add storage_type = "gp3" to aws_db_instance):

locals {
  rds_storage_type = "gp3"   # was "gp2" — saves ~$0.70/month per 20 GB
}

# ---------------------------------------------------------------------------
# CloudWatch Log retention — prevent unbounded log storage costs
# (Supplements resource-optimization.tf which covers indexer/api log groups)
# ---------------------------------------------------------------------------

resource "aws_cloudwatch_log_group" "prometheus" {
  name              = "/stellarescrow/${var.environment}/prometheus"
  retention_in_days = 14
}

resource "aws_cloudwatch_log_group" "grafana" {
  name              = "/stellarescrow/${var.environment}/grafana"
  retention_in_days = 14
}

# ---------------------------------------------------------------------------
# S3 backup bucket — abort incomplete multipart uploads (hidden cost)
# ---------------------------------------------------------------------------

resource "aws_s3_bucket_lifecycle_configuration" "backup_cleanup" {
  bucket = "stellarescrow-${var.environment}-backups"

  rule {
    id     = "abort-incomplete-multipart"
    status = "Enabled"

    abort_incomplete_multipart_upload {
      days_after_initiation = 7
    }
  }

  rule {
    id     = "transition-to-ia"
    status = "Enabled"

    transition {
      days          = 30
      storage_class = "STANDARD_IA"
    }

    transition {
      days          = 90
      storage_class = "GLACIER_IR"
    }

    expiration {
      days = 365
    }
  }
}

# ---------------------------------------------------------------------------
# Cost allocation tags — required for per-service cost breakdown in reports
# ---------------------------------------------------------------------------

resource "aws_resourcegroups_group" "stellarescrow" {
  name = "stellarescrow-${var.environment}"

  resource_query {
    query = jsonencode({
      ResourceTypeFilters = ["AWS::AllSupported"]
      TagFilters = [{
        Key    = "Project"
        Values = ["StellarEscrow"]
      }, {
        Key    = "Environment"
        Values = [var.environment]
      }]
    })
  }
}
