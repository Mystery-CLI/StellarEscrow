variable "name_prefix" { type = string }
variable "vpc_id" { type = string }
variable "private_subnet_ids" { type = list(string) }
variable "api_security_group_id" { type = string }

variable "instance_class" {
  type    = string
  default = "db.t3.micro"
}
variable "allocated_storage_gb" { type = number }
variable "max_allocated_storage_gb" {
  type    = number
  default = 0
}
variable "engine_version" {
  type    = string
  default = "15.6"
}
variable "db_name" { type = string }
variable "db_username" {
  type      = string
  sensitive = true
}
variable "db_password" {
  type      = string
  sensitive = true
}
variable "multi_az" {
  type    = bool
  default = false
}
variable "create_read_replica" {
  type    = bool
  default = false
}
variable "replica_instance_class" {
  type    = string
  default = "db.t3.micro"
}
variable "backup_retention_days" {
  type    = number
  default = 1
}
variable "backup_window" {
  type    = string
  default = "03:00-04:00"
}
variable "maintenance_window" {
  type    = string
  default = "Mon:04:00-Mon:05:00"
}
variable "deletion_protection" {
  type    = bool
  default = false
}
variable "alarm_sns_arn" {
  type    = string
  default = ""
}
variable "cpu_alarm_threshold" {
  type    = number
  default = 80
}
variable "free_storage_alarm_gb" {
  type    = number
  default = 5
}
variable "connections_alarm" {
  type    = number
  default = 200
}
