variable "name_prefix" {
  description = "Prefix for all resource names"
  type        = string
}

variable "environment" {
  description = "Deployment environment (e.g., staging, production)"
  type        = string
}

variable "vpc_id" {
  description = "VPC ID where the ALB will be deployed"
  type        = string
}

variable "public_subnet_ids" {
  description = "List of public subnet IDs for the ALB"
  type        = list(string)
}

variable "private_subnet_ids" {
  description = "List of private subnet IDs for ECS task routing"
  type        = list(string)
}

# ── Health Checks ───────────────────────────────────────────────────────────

variable "container_port" {
  description = "Port the container is listening on"
  type        = number
  default     = 3000
}

variable "health_check_path" {
  description = "Path for ALB health checks"
  type        = string
  default     = "/health"
}

variable "health_check_interval" {
  description = "Number of seconds between health checks"
  type        = number
  default     = 30
}

variable "health_check_timeout" {
  description = "Number of seconds to wait for a health check response"
  type        = number
  default     = 5
}

variable "healthy_threshold" {
  description = "Number of consecutive successes to mark a target healthy"
  type        = number
  default     = 2
}

variable "unhealthy_threshold" {
  description = "Number of consecutive failures to mark a target unhealthy"
  type        = number
  default     = 3
}

# ── Session & Protection ───────────────────────────────────────────────────

variable "enable_stickiness" {
  description = "Enable session affinity (sticky sessions)"
  type        = bool
  default     = false
}

variable "stickiness_duration" {
  description = "Duration of the sticky session in seconds"
  type        = number
  default     = 86400
}

variable "enable_deletion_protection" {
  description = "Enable deletion protection for the ALB"
  type        = bool
  default     = false
}

variable "certificate_arn" {
  description = "ARN of the SSL certificate for the HTTPS listener"
  type        = string
  default     = ""
}

variable "alarm_sns_arn" {
  description = "SNS topic ARN for CloudWatch alarms"
  type        = string
  default     = ""
}

# ── Autoscaling ─────────────────────────────────────────────────────────────

variable "ecs_cluster_name" {
  description = "Name of the ECS cluster"
  type        = string
}

variable "ecs_service_name" {
  description = "Name of the ECS service"
  type        = string
}

variable "autoscaling_min_capacity" {
  description = "Minimum number of tasks"
  type        = number
  default     = 1
}

variable "autoscaling_max_capacity" {
  description = "Maximum number of tasks"
  type        = number
  default     = 4
}

variable "scale_out_cpu_threshold" {
  description = "CPU threshold to trigger scale out (%)"
  type        = number
  default     = 70
}

variable "scale_in_cpu_threshold" {
  description = "CPU threshold to trigger scale in (%)"
  type        = number
  default     = 30
}

variable "scale_out_request_threshold" {
  description = "Requests-per-target threshold to trigger scale out"
  type        = number
  default     = 1000
}

variable "aws_region" {
  description = "AWS region for regional resources"
  type        = string
  default     = "us-east-1"
}
