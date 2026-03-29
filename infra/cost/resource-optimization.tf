# Resource Optimization — cost-saving configurations
# Applies gp3 storage, log retention, ECR lifecycle, S3 intelligent tiering

# ---------------------------------------------------------------------------
# CloudWatch Log Groups — enforce retention to prevent unbounded cost
# ---------------------------------------------------------------------------

resource "aws_cloudwatch_log_group" "indexer" {
  name              = "/stellarescrow/${var.environment}/indexer"
  retention_in_days = var.environment == "production" ? 30 : 14
}

resource "aws_cloudwatch_log_group" "api" {
  name              = "/stellarescrow/${var.environment}/api"
  retention_in_days = var.environment == "production" ? 30 : 14
}

resource "aws_cloudwatch_log_group" "ecs_exec" {
  name              = "/stellarescrow/${var.environment}/ecs-exec"
  retention_in_days = 7
}

# ---------------------------------------------------------------------------
# ECR Lifecycle Policies — keep only last 5 images per repo
# ---------------------------------------------------------------------------

resource "aws_ecr_lifecycle_policy" "indexer" {
  repository = "stellarescrow-${var.environment}-indexer"

  policy = jsonencode({
    rules = [{
      rulePriority = 1
      description  = "Keep last 5 images"
      selection = {
        tagStatus   = "any"
        countType   = "imageCountMoreThan"
        countNumber = 5
      }
      action = { type = "expire" }
    }]
  })
}

resource "aws_ecr_lifecycle_policy" "api" {
  repository = "stellarescrow-${var.environment}-api"

  policy = jsonencode({
    rules = [{
      rulePriority = 1
      description  = "Keep last 5 images"
      selection = {
        tagStatus   = "any"
        countType   = "imageCountMoreThan"
        countNumber = 5
      }
      action = { type = "expire" }
    }]
  })
}

# ---------------------------------------------------------------------------
# S3 Backup Bucket — Intelligent-Tiering + lifecycle to Glacier
# ---------------------------------------------------------------------------

resource "aws_s3_bucket_intelligent_tiering_configuration" "backups" {
  bucket = "stellarescrow-backups"
  name   = "EntireBucket"

  tiering {
    access_tier = "DEEP_ARCHIVE_ACCESS"
    days        = 180
  }

  tiering {
    access_tier = "ARCHIVE_ACCESS"
    days        = 90
  }
}

resource "aws_s3_bucket_lifecycle_configuration" "backups" {
  bucket = "stellarescrow-backups"

  rule {
    id     = "expire-old-backups"
    status = "Enabled"

    expiration {
      days = 90
    }

    noncurrent_version_expiration {
      noncurrent_days = 30
    }
  }
}

# ---------------------------------------------------------------------------
# Variable declarations (referenced above)
# ---------------------------------------------------------------------------

variable "environment" {
  description = "Deployment environment"
  type        = string
  default     = "production"
}
