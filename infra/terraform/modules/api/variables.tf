variable "name_prefix" {
  type = string
}

variable "environment" {
  type = string
}

variable "vpc_id" {
  type = string
}

variable "public_subnet_ids" {
  type = list(string)
}

variable "private_subnet_ids" {
  type = list(string)
}

variable "api_image" {
  type = string
}

variable "desired_count" {
  type    = number
  default = 1
}

variable "cpu" {
  type    = number
  default = 256
}

variable "memory" {
  type    = number
  default = 512
}

variable "container_port" {
  type    = number
  default = 3000
}

variable "aws_region" {
  type = string
}

variable "db_secret_arn" {
  type      = string
  sensitive = true
}

variable "stellar_network" {
  type = string
}

variable "stellar_contract_id" {
  type = string
}

variable "stellar_horizon_url" {
  type = string
}

variable "enable_deletion_protection" {
  type    = bool
  default = false
}

# Injected from load_balancer module
variable "ecs_security_group_id" {
  type = string
}

variable "api_target_group_arn" {
  type = string
}
