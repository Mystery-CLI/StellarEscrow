variable "name_prefix" {
  description = "Prefix for all resource names"
  type        = string
}

variable "vpc_id" {
  description = "VPC ID where the database will be deployed"
  type        = string
}

variable "private_subnet_ids" {
  description = "List of private subnet IDs for the RDS subnet group"
  type        = list(string)
}

variable "api_security_group_id" {
  description = "Security group ID of the API tasks to allow inbound traffic"
  type        = string
}

# ── Performance & Scaling ───────────────────────────────────────────────────

variable "instance_class" {
  description = "RDS instance class"
  type        = string
  default     = "db.t3.micro"
}

variable "allocated_storage_gb" {
  description = "Allocated storage in GB"
  type        = number
}

variable "max_allocated_storage_gb" {
  description = "Maximum storage limit for autoscaling in GB (0 to disable)"
  type        = number
  default     = 0
}

variable "engine_version" {
  description = "PostgreSQL engine version"
  type        = string
  default     = "15.6"
}

variable "db_name" {
  description = "Name of the initial database"
  type        = string
}

variable "db_username" {
  description = "Master username"
  type        = string
  sensitive   = true
}

variable "db_password" {
  description = "Master password"
  type        = string
  sensitive   = true
}

# ── High Availability ────────────────────────────────────────────────────────

variable "multi_az" {
  description = "Enable Multi-AZ deployment"
  type        = bool
  default     = false
}

variable "create_read_replica" {
  description = "Create a read-only replica"
  type        = bool
  default     = false
}

variable "replica_instance_class" {
  description = "RDS instance class for the read replica"
  type        = string
  default     = "db.t3.micro"
}

# ── Maintenance & Monitoring ───────────────────────────────────────────────

variable "backup_retention_days" {
  description = "Number of days to retain backups"
  type        = number
  default     = 1
}

variable "backup_window" {
  description = "Daily backup window (HH:MM-HH:MM)"
  type        = string
  default     = "03:00-04:00"
}

variable "maintenance_window" {
  description = "Weekly maintenance window"
  type        = string
  default     = "Mon:04:00-Mon:05:00"
}

variable "deletion_protection" {
  description = "Enable deletion protection"
  type        = bool
  default     = false
}

variable "alarm_sns_arn" {
  description = "SNS topic ARN for CloudWatch alarms"
  type        = string
  default     = ""
}

variable "cpu_alarm_threshold" {
  description = "CPU utilization threshold for alarms (%)"
  type        = number
  default     = 80
}

variable "free_storage_alarm_gb" {
  description = "Free storage threshold for alarms (GB)"
  type        = number
  default     = 5
}

variable "connections_alarm" {
  description = "Database connections threshold for alarms"
  type        = number
  default     = 200
}

variable "aws_region" {
  description = "AWS region for regional resources"
  type        = string
  default     = "us-east-1"
}
