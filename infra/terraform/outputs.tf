output "vpc_id" {
  value = module.networking.vpc_id
}

output "api_url" {
  value = module.load_balancer.alb_dns_name
}

output "alb_arn" {
  value = module.load_balancer.alb_arn
}

output "db_endpoint" {
  value     = module.database.endpoint
  sensitive = true
}

output "db_replica_endpoint" {
  value     = module.database.replica_endpoint
  sensitive = true
}

output "ecr_repository_url" {
  value = module.api.ecr_repository_url
}

output "ecs_cluster_name" {
  value = module.api.ecs_cluster_name
}

output "lb_dashboard_name" {
  value = module.load_balancer.dashboard_name
}

output "db_dashboard_name" {
  value = module.database.dashboard_name
}

output "infra_version" {
  value = local.infra_version
}
