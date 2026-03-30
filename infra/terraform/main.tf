terraform {
  required_version = ">= 1.7.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "5.40.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "3.6.0"
    }
  }

  # Remote state — S3 backend with DynamoDB locking.
  # Bucket and table are bootstrapped by infra/bootstrap/main.tf.
  backend "s3" {
    bucket         = "stellarescrow-tfstate"
    # Key is provided via -backend-config during init or hardcoded to a base path.
    # example: terraform init -backend-config="key=stellar-escrow/staging/terraform.tfstate"
    key            = "stellar-escrow/terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "stellarescrow-tfstate-lock"
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project      = "StellarEscrow"
      Environment  = var.environment
      ManagedBy    = "Terraform"
      Version      = var.app_version
      InfraVersion = local.infra_version
    }
  }
}
