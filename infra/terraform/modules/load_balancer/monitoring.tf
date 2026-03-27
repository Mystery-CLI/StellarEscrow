# ── CloudWatch Alarms ─────────────────────────────────────────────────────────

locals {
  alarm_actions = var.alarm_sns_arn != "" ? [var.alarm_sns_arn] : []
}

# 5xx error rate > 5% for 5 minutes
resource "aws_cloudwatch_metric_alarm" "alb_5xx" {
  alarm_name          = "${var.name_prefix}-alb-5xx-high"
  alarm_description   = "ALB 5xx error rate exceeded 5% for 5 minutes"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 5
  threshold           = 5
  treat_missing_data  = "notBreaching"

  metric_query {
    id          = "error_rate"
    expression  = "errors / MAX([errors, requests]) * 100"
    label       = "5xx Error Rate (%)"
    return_data = true
  }

  metric_query {
    id = "errors"
    metric {
      namespace   = "AWS/ApplicationELB"
      metric_name = "HTTPCode_ELB_5XX_Count"
      dimensions  = { LoadBalancer = aws_lb.main.arn_suffix }
      period      = 60
      stat        = "Sum"
    }
  }

  metric_query {
    id = "requests"
    metric {
      namespace   = "AWS/ApplicationELB"
      metric_name = "RequestCount"
      dimensions  = { LoadBalancer = aws_lb.main.arn_suffix }
      period      = 60
      stat        = "Sum"
    }
  }

  alarm_actions = local.alarm_actions
  ok_actions    = local.alarm_actions
}

# Target response time p99 > 2 seconds
resource "aws_cloudwatch_metric_alarm" "alb_latency_p99" {
  alarm_name          = "${var.name_prefix}-alb-latency-p99"
  alarm_description   = "ALB p99 response time > 2s for 3 consecutive minutes"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 3
  threshold           = 2
  treat_missing_data  = "notBreaching"

  namespace   = "AWS/ApplicationELB"
  metric_name = "TargetResponseTime"
  dimensions  = { LoadBalancer = aws_lb.main.arn_suffix }
  period      = 60
  statistic   = "p99"
  extended_statistic = "p99"

  alarm_actions = local.alarm_actions
  ok_actions    = local.alarm_actions
}

# Unhealthy host count > 0 for 2 minutes
resource "aws_cloudwatch_metric_alarm" "unhealthy_hosts" {
  alarm_name          = "${var.name_prefix}-unhealthy-hosts"
  alarm_description   = "One or more ECS tasks are failing health checks"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 2
  threshold           = 0
  treat_missing_data  = "notBreaching"

  namespace   = "AWS/ApplicationELB"
  metric_name = "UnHealthyHostCount"
  dimensions = {
    LoadBalancer = aws_lb.main.arn_suffix
    TargetGroup  = aws_lb_target_group.api.arn_suffix
  }
  period    = 60
  statistic = "Maximum"

  alarm_actions = local.alarm_actions
  ok_actions    = local.alarm_actions
}

# ECS CPU utilisation > scale-out threshold
resource "aws_cloudwatch_metric_alarm" "ecs_cpu_high" {
  alarm_name          = "${var.name_prefix}-ecs-cpu-high"
  alarm_description   = "ECS service CPU utilisation above scale-out threshold"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 2
  threshold           = var.scale_out_cpu_threshold
  treat_missing_data  = "notBreaching"

  namespace   = "AWS/ECS"
  metric_name = "CPUUtilization"
  dimensions = {
    ClusterName = var.ecs_cluster_name
    ServiceName = var.ecs_service_name
  }
  period    = 60
  statistic = "Average"

  alarm_actions = concat(local.alarm_actions, [aws_appautoscaling_policy.scale_out_cpu.arn])
  ok_actions    = concat(local.alarm_actions, [aws_appautoscaling_policy.scale_in_cpu.arn])
}

# ── CloudWatch Dashboard ──────────────────────────────────────────────────────

resource "aws_cloudwatch_dashboard" "lb" {
  dashboard_name = "${var.name_prefix}-load-balancer"

  dashboard_body = jsonencode({
    widgets = [
      {
        type   = "metric"
        x      = 0
        y      = 0
        width  = 12
        height = 6
        properties = {
          title  = "Request Count"
          region = "us-east-1"
          metrics = [
            ["AWS/ApplicationELB", "RequestCount",
              "LoadBalancer", aws_lb.main.arn_suffix,
              { stat = "Sum", period = 60, label = "Total Requests" }]
          ]
          view = "timeSeries"
        }
      },
      {
        type   = "metric"
        x      = 12
        y      = 0
        width  = 12
        height = 6
        properties = {
          title  = "HTTP Error Rates"
          region = "us-east-1"
          metrics = [
            ["AWS/ApplicationELB", "HTTPCode_ELB_4XX_Count",
              "LoadBalancer", aws_lb.main.arn_suffix,
              { stat = "Sum", period = 60, label = "4xx" }],
            ["AWS/ApplicationELB", "HTTPCode_ELB_5XX_Count",
              "LoadBalancer", aws_lb.main.arn_suffix,
              { stat = "Sum", period = 60, label = "5xx" }]
          ]
          view = "timeSeries"
        }
      },
      {
        type   = "metric"
        x      = 0
        y      = 6
        width  = 12
        height = 6
        properties = {
          title  = "Target Response Time (p50 / p99)"
          region = "us-east-1"
          metrics = [
            ["AWS/ApplicationELB", "TargetResponseTime",
              "LoadBalancer", aws_lb.main.arn_suffix,
              { stat = "p50", period = 60, label = "p50" }],
            ["AWS/ApplicationELB", "TargetResponseTime",
              "LoadBalancer", aws_lb.main.arn_suffix,
              { stat = "p99", period = 60, label = "p99" }]
          ]
          view = "timeSeries"
        }
      },
      {
        type   = "metric"
        x      = 12
        y      = 6
        width  = 12
        height = 6
        properties = {
          title  = "Healthy / Unhealthy Hosts"
          region = "us-east-1"
          metrics = [
            ["AWS/ApplicationELB", "HealthyHostCount",
              "LoadBalancer", aws_lb.main.arn_suffix,
              "TargetGroup", aws_lb_target_group.api.arn_suffix,
              { stat = "Average", period = 60, label = "Healthy" }],
            ["AWS/ApplicationELB", "UnHealthyHostCount",
              "LoadBalancer", aws_lb.main.arn_suffix,
              "TargetGroup", aws_lb_target_group.api.arn_suffix,
              { stat = "Average", period = 60, label = "Unhealthy", color = "#d62728" }]
          ]
          view = "timeSeries"
        }
      },
      {
        type   = "metric"
        x      = 0
        y      = 12
        width  = 12
        height = 6
        properties = {
          title  = "ECS CPU Utilisation"
          region = "us-east-1"
          metrics = [
            ["AWS/ECS", "CPUUtilization",
              "ClusterName", var.ecs_cluster_name,
              "ServiceName", var.ecs_service_name,
              { stat = "Average", period = 60, label = "CPU %" }]
          ]
          annotations = {
            horizontal = [
              { value = var.scale_out_cpu_threshold, label = "Scale-out threshold", color = "#ff7f0e" },
              { value = var.scale_in_cpu_threshold,  label = "Scale-in threshold",  color = "#2ca02c" }
            ]
          }
          view = "timeSeries"
        }
      },
      {
        type   = "metric"
        x      = 12
        y      = 12
        width  = 12
        height = 6
        properties = {
          title  = "ECS Task Count"
          region = "us-east-1"
          metrics = [
            ["ECS/ContainerInsights", "RunningTaskCount",
              "ClusterName", var.ecs_cluster_name,
              "ServiceName", var.ecs_service_name,
              { stat = "Average", period = 60, label = "Running Tasks" }]
          ]
          annotations = {
            horizontal = [
              { value = var.autoscaling_min_capacity, label = "Min", color = "#2ca02c" },
              { value = var.autoscaling_max_capacity, label = "Max", color = "#d62728" }
            ]
          }
          view = "timeSeries"
        }
      }
    ]
  })
}
