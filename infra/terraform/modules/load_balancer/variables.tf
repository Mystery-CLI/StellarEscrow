variable "name_prefix" { type = string }
variable "environment" { type = string }
variable "vpc_id" { type = string }
variable "public_subnet_ids" { type = list(string) }
variable "private_subnet_ids" { type = list(string) }

variable "container_port" {
  type    = number
  default = 3000
}
variable "health_check_path" {
  type    = string
  default = "/health"
}
variable "health_check_interval" {
  type    = number
  default = 30
}
variable "health_check_timeout" {
  type    = number
  default = 5
}
variable "healthy_threshold" {
  type    = number
  default = 2
}
variable "unhealthy_threshold" {
  type    = number
  default = 3
}
variable "enable_stickiness" {
  type    = bool
  default = false
}
variable "stickiness_duration" {
  type    = number
  default = 86400
}
variable "enable_deletion_protection" {
  type    = bool
  default = false
}
variable "certificate_arn" {
  type    = string
  default = ""
}
variable "alarm_sns_arn" {
  type    = string
  default = ""
}

# Autoscaling
variable "ecs_cluster_name" { type = string }
variable "ecs_service_name" { type = string }
variable "autoscaling_min_capacity" {
  type    = number
  default = 1
}
variable "autoscaling_max_capacity" {
  type    = number
  default = 4
}
variable "scale_out_cpu_threshold" {
  type    = number
  default = 70
}
variable "scale_in_cpu_threshold" {
  type    = number
  default = 30
}
variable "scale_out_request_threshold" {
  type    = number
  default = 1000
}
