# ── Network Outputs ─────────────────────────────────────────────────────────

output "vpc_id" {
  description = "The ID of the VPC"
  value       = module.networking.vpc_id
}

output "api_url" {
  description = "The DNS name of the Load Balancer"
  value       = module.load_balancer.alb_dns_name
}

output "alb_arn" {
  description = "The ARN of the Load Balancer"
  value       = module.load_balancer.alb_arn
}

# ── Database Outputs ────────────────────────────────────────────────────────

output "db_endpoint" {
  description = "The connection endpoint for the primary database"
  value       = module.database.endpoint
  sensitive   = true
}

output "db_replica_endpoint" {
  description = "The connection endpoint for the read replica"
  value       = module.database.replica_endpoint
  sensitive   = true
}

# ── API & Container Outputs ──────────────────────────────────────────────────

output "ecr_repository_url" {
  description = "The URL of the ECR repository"
  value       = module.api.ecr_repository_url
}

output "ecs_cluster_name" {
  description = "The name of the ECS cluster"
  value       = module.api.ecs_cluster_name
}

# ── Observability Outputs ────────────────────────────────────────────────────

output "lb_dashboard_name" {
  description = "The name of the CloudWatch dashboard for the Load Balancer"
  value       = module.load_balancer.dashboard_name
}

output "db_dashboard_name" {
  description = "The name of the CloudWatch dashboard for the Database"
  value       = module.database.dashboard_name
}

# ── Metadata Outputs ────────────────────────────────────────────────────────

output "infra_version" {
  description = "Current version of the infrastructure code"
  value       = local.infra_version
}
