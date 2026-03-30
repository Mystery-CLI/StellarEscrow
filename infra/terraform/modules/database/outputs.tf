output "endpoint" {
  value     = aws_db_instance.primary.endpoint
  sensitive = true
}
output "replica_endpoint" {
  value     = var.create_read_replica ? aws_db_instance.replica[0].endpoint : ""
  sensitive = true
}
output "db_name" { value = aws_db_instance.primary.db_name }
output "security_group_id" { value = aws_security_group.db.id }
output "dashboard_name" { value = aws_cloudwatch_dashboard.db.dashboard_name }
