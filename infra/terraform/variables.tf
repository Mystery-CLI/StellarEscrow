variable "environment" {
  description = "Deployment environment (development | staging | production)"
  type        = string
  validation {
    condition     = contains(["development", "staging", "production"], var.environment)
    error_message = "environment must be development, staging, or production."
  }
}

variable "aws_region" {
  type    = string
  default = "us-east-1"
}

variable "app_version" {
  type    = string
  default = "latest"
}

# ── Networking ────────────────────────────────────────────────────────────────
variable "vpc_cidr" {
  type    = string
  default = "10.0.0.0/16"
}

variable "availability_zones" {
  type    = list(string)
  default = ["us-east-1a", "us-east-1b"]
}

# ── Database ──────────────────────────────────────────────────────────────────
variable "db_instance_class" {
  description = "RDS primary instance class"
  type        = string
  default     = "db.t3.micro"
}

variable "db_name" {
  type    = string
  default = "stellar_escrow"
}

variable "db_username" {
  type      = string
  default   = "indexer"
  sensitive = true
}

variable "db_password" {
  type      = string
  sensitive = true
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
  type    = string
  default = "03:00-04:00"
}

variable "db_maintenance_window" {
  type    = string
  default = "Mon:04:00-Mon:05:00"
}

# ── API ───────────────────────────────────────────────────────────────────────
variable "api_image" {
  type    = string
  default = "stellarescrow/api:latest"
}

variable "api_cpu" {
  type    = number
  default = 256
}

variable "api_memory" {
  type    = number
  default = 512
}

variable "api_container_port" {
  type    = number
  default = 3000
}

# ── Load Balancer ─────────────────────────────────────────────────────────────
variable "certificate_arn" {
  type    = string
  default = ""
}

variable "alarm_sns_arn" {
  type    = string
  default = ""
}

# ── Stellar ───────────────────────────────────────────────────────────────────
variable "stellar_network" {
  type    = string
  default = "testnet"
}

variable "stellar_contract_id" {
  type    = string
  default = ""
}

variable "stellar_horizon_url" {
  type    = string
  default = "https://horizon-testnet.stellar.org"
}
