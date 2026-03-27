output "alb_arn" { value = aws_lb.main.arn }
output "alb_dns_name" { value = aws_lb.main.dns_name }
output "alb_zone_id" { value = aws_lb.main.zone_id }
output "alb_sg_id" { value = aws_security_group.alb.id }
output "ecs_sg_id" { value = aws_security_group.ecs_tasks.id }
output "api_target_group_arn" { value = aws_lb_target_group.api.arn }
output "ws_target_group_arn" { value = aws_lb_target_group.ws.arn }
output "http_listener_arn" { value = aws_lb_listener.http.arn }
output "https_listener_arn" { value = var.certificate_arn != "" ? aws_lb_listener.https[0].arn : "" }
output "dashboard_name" { value = aws_cloudwatch_dashboard.lb.dashboard_name }
