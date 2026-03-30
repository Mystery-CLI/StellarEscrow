variable "environment" {
  description = "Deployment environment (development | staging | production)"
  type        = string
  validation {
    condition     = contains(["development", "staging", "production"], var.environment)
    error_message = "environment must be development, staging, or production."
  }
}

variable "aws_region" {
  description = "AWS region for all resources"
  type        = string
  default     = "us-east-1"
}

variable "app_version" {
  description = "Application version tag for resources"
  type        = string
  default     = "latest"
}

# ── Networking ────────────────────────────────────────────────────────────────

variable "vpc_cidr" {
  description = "CIDR block for the VPC"
  type        = string
  default     = "10.0.0.0/16"
  validation {
    condition     = can(cidrnetmask(var.vpc_cidr))
    error_message = "vpc_cidr must be a valid CIDR block."
  }
}

variable "availability_zones" {
  description = "List of availability zones for the VPC"
  type        = list(string)
  default     = ["us-east-1a", "us-east-1b"]
}

# ── Database ──────────────────────────────────────────────────────────────────

variable "db_instance_class" {
  description = "RDS primary instance class"
  type        = string
  default     = "db.t3.micro"
  validation {
    condition     = can(regex("^db\\.", var.db_instance_class))
    error_message = "db_instance_class must start with 'db.'."
  }
}

variable "db_name" {
  description = "Initial database name"
  type        = string
  default     = "stellar_escrow"
}

variable "db_username" {
  description = "Database master username"
  type        = string
  default     = "indexer"
  sensitive   = true
}

variable "db_password" {
  description = "Database master password"
  type        = string
  sensitive   = true
}

variable "db_allocated_storage_gb" {
  description = "Initial RDS storage in GB"
  type        = number
  default     = 20
}

variable "db_max_allocated_storage_gb" {
  description = "Storage autoscaling ceiling in GB (0 = disabled)"
  type        = number
  default     = 100
}

variable "db_engine_version" {
  description = "PostgreSQL engine version"
  type        = string
  default     = "15.6"
}

variable "db_backup_window" {
  description = "Daily backup window (HH:MM-HH:MM)"
  type        = string
  default     = "03:00-04:00"
}

variable "db_maintenance_window" {
  description = "Weekly maintenance window"
  type        = string
  default     = "Mon:04:00-Mon:05:00"
}

# ── API ───────────────────────────────────────────────────────────────────────

variable "api_image" {
  description = "Docker image for the API service"
  type        = string
  default     = "stellarescrow/api:latest"
}

variable "api_cpu" {
  description = "Fargate task CPU units"
  type        = number
  default     = 256
  validation {
    condition     = contains([256, 512, 1024, 2048, 4096], var.api_cpu)
    error_message = "api_cpu must be one of 256, 512, 1024, 2048, or 4096."
  }
}

variable "api_memory" {
  description = "Fargate task memory (MB)"
  type        = number
  default     = 512
}

variable "api_container_port" {
  description = "Internal port the API container listens on"
  type        = number
  default     = 3000
}

# ── Load Balancer ─────────────────────────────────────────────────────────────

variable "certificate_arn" {
  description = "ACM certificate ARN for HTTPS"
  type        = string
  default     = ""
}

variable "alarm_sns_arn" {
  description = "SNS topic ARN for infrastructure alerts"
  type        = string
  default     = ""
}

# ── Stellar ───────────────────────────────────────────────────────────────────

variable "stellar_network" {
  description = "Stellar network to connect to (testnet | public)"
  type        = string
  default     = "testnet"
}

variable "stellar_contract_id" {
  description = "Stellar Escrow contract ID"
  type        = string
  default     = ""
}

variable "stellar_horizon_url" {
  description = "URL for the Stellar Horizon server"
  type        = string
  default     = "https://horizon-testnet.stellar.org"
}
