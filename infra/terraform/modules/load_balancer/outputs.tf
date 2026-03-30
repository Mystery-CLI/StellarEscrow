output "alb_arn" {
  description = "ARN of the Application Load Balancer"
  value       = aws_lb.main.arn
}

output "alb_dns_name" {
  description = "DNS name of the Application Load Balancer"
  value       = aws_lb.main.dns_name
}

output "alb_zone_id" {
  description = "Canonical Hosted Zone ID of the ALB (for Route53)"
  value       = aws_lb.main.zone_id
}

output "alb_sg_id" {
  description = "Security group ID of the ALB"
  value       = aws_security_group.alb.id
}

output "ecs_sg_id" {
  description = "Security group ID for the ECS tasks (allows inbound from ALB)"
  value       = aws_security_group.ecs_tasks.id
}

output "api_target_group_arn" {
  description = "ARN of the primary API target group"
  value       = aws_lb_target_group.api.arn
}

output "ws_target_group_arn" {
  description = "ARN of the WebSocket target group"
  value       = aws_lb_target_group.ws.arn
}

output "http_listener_arn" {
  description = "ARN of the HTTP listener"
  value       = aws_lb_listener.http.arn
}

output "https_listener_arn" {
  description = "ARN of the HTTPS listener (if created)"
  value       = var.certificate_arn != "" ? aws_lb_listener.https[0].arn : ""
}

output "dashboard_name" {
  description = "Name of the CloudWatch dashboard for the LB"
  value       = aws_cloudwatch_dashboard.lb.dashboard_name
}
