# Bootstrap — creates the S3 bucket and DynamoDB table used by the Terraform
# remote backend. Run once per AWS account before any other `terraform apply`.
#
# Usage:
#   cd infra/bootstrap
#   terraform init
#   terraform apply

terraform {
  required_version = ">= 1.7.0"
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.40" }
  }
  # Bootstrap uses local state intentionally — it manages the remote backend itself.
}

provider "aws" {
  region = var.aws_region
  default_tags {
    tags = {
      Project   = "StellarEscrow"
      ManagedBy = "Terraform-Bootstrap"
    }
  }
}

variable "aws_region" {
  type    = string
  default = "us-east-1"
}

resource "aws_s3_bucket" "tfstate" {
  bucket = "stellarescrow-tfstate"
  # Prevent accidental deletion of state
  lifecycle { prevent_destroy = true }
}

resource "aws_s3_bucket_versioning" "tfstate" {
  bucket = aws_s3_bucket.tfstate.id
  versioning_configuration { status = "Enabled" }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "tfstate" {
  bucket = aws_s3_bucket.tfstate.id
  rule {
    apply_server_side_encryption_by_default { sse_algorithm = "AES256" }
  }
}

resource "aws_s3_bucket_public_access_block" "tfstate" {
  bucket                  = aws_s3_bucket.tfstate.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_dynamodb_table" "tfstate_lock" {
  name         = "stellarescrow-tfstate-lock"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "LockID"

  attribute {
    name = "LockID"
    type = "S"
  }

  lifecycle { prevent_destroy = true }
}

output "state_bucket" { value = aws_s3_bucket.tfstate.bucket }
output "lock_table"   { value = aws_dynamodb_table.tfstate_lock.name }
