terraform {
  required_version = ">= 1.7.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.40"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }

  # Remote state — S3 backend with DynamoDB locking.
  # Bucket and table are bootstrapped by infra/bootstrap/main.tf.
  backend "s3" {
    bucket         = "stellarescrow-tfstate"
    key            = "stellar-escrow/${var.environment}/terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "stellarescrow-tfstate-lock"
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "StellarEscrow"
      Environment = var.environment
      ManagedBy   = "Terraform"
      Version     = var.app_version
    }
  }
}
