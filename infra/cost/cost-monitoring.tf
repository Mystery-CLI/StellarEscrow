# Cost Monitoring — AWS Budgets + CloudWatch billing alerts
# Apply with: terraform apply -target=module.cost_monitoring

terraform {
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.0" }
  }
}

variable "alert_email" {
  description = "Email address for cost alerts"
  type        = string
  default     = "ops@stellarescrow.io"
}

variable "monthly_budget_usd" {
  description = "Monthly budget threshold in USD"
  type        = number
  default     = 150
}

variable "environment" {
  description = "Deployment environment"
  type        = string
  default     = "production"
}

# ---------------------------------------------------------------------------
# Monthly budget with email + SNS alerts
# ---------------------------------------------------------------------------

resource "aws_budgets_budget" "monthly" {
  name         = "stellarescrow-${var.environment}-monthly"
  budget_type  = "COST"
  limit_amount = tostring(var.monthly_budget_usd)
  limit_unit   = "USD"
  time_unit    = "MONTHLY"

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 80
    threshold_type             = "PERCENTAGE"
    notification_type          = "ACTUAL"
    subscriber_email_addresses = [var.alert_email]
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 100
    threshold_type             = "PERCENTAGE"
    notification_type          = "ACTUAL"
    subscriber_email_addresses = [var.alert_email]
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 110
    threshold_type             = "PERCENTAGE"
    notification_type          = "FORECASTED"
    subscriber_email_addresses = [var.alert_email]
  }
}

# ---------------------------------------------------------------------------
# Per-service budgets
# ---------------------------------------------------------------------------

resource "aws_budgets_budget" "fargate" {
  name         = "stellarescrow-${var.environment}-fargate"
  budget_type  = "COST"
  limit_amount = "40"
  limit_unit   = "USD"
  time_unit    = "MONTHLY"

  cost_filter {
    name   = "Service"
    values = ["Amazon Elastic Container Service"]
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 90
    threshold_type             = "PERCENTAGE"
    notification_type          = "ACTUAL"
    subscriber_email_addresses = [var.alert_email]
  }
}

resource "aws_budgets_budget" "rds" {
  name         = "stellarescrow-${var.environment}-rds"
  budget_type  = "COST"
  limit_amount = "30"
  limit_unit   = "USD"
  time_unit    = "MONTHLY"

  cost_filter {
    name   = "Service"
    values = ["Amazon Relational Database Service"]
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 90
    threshold_type             = "PERCENTAGE"
    notification_type          = "ACTUAL"
    subscriber_email_addresses = [var.alert_email]
  }
}

resource "aws_budgets_budget" "networking" {
  name         = "stellarescrow-${var.environment}-networking"
  budget_type  = "COST"
  limit_amount = "60"
  limit_unit   = "USD"
  time_unit    = "MONTHLY"

  cost_filter {
    name   = "Service"
    values = ["Amazon Virtual Private Cloud", "Amazon EC2"]
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 90
    threshold_type             = "PERCENTAGE"
    notification_type          = "ACTUAL"
    subscriber_email_addresses = [var.alert_email]
  }
}

# ---------------------------------------------------------------------------
# SNS topic for programmatic alerts (webhook / Slack integration)
# ---------------------------------------------------------------------------

resource "aws_sns_topic" "cost_alerts" {
  name = "stellarescrow-${var.environment}-cost-alerts"
}

resource "aws_sns_topic_subscription" "cost_email" {
  topic_arn = aws_sns_topic.cost_alerts.arn
  protocol  = "email"
  endpoint  = var.alert_email
}

# ---------------------------------------------------------------------------
# CloudWatch anomaly detection on estimated charges
# ---------------------------------------------------------------------------

resource "aws_cloudwatch_metric_alarm" "billing_anomaly" {
  alarm_name          = "stellarescrow-${var.environment}-billing-anomaly"
  comparison_operator = "GreaterThanUpperThreshold"
  evaluation_periods  = 1
  threshold_metric_id = "ad1"
  alarm_description   = "Billing anomaly detected — charges exceed expected range"
  alarm_actions       = [aws_sns_topic.cost_alerts.arn]

  metric_query {
    id          = "m1"
    return_data = false
    metric {
      metric_name = "EstimatedCharges"
      namespace   = "AWS/Billing"
      period      = 86400
      stat        = "Maximum"
      dimensions  = { Currency = "USD" }
    }
  }

  metric_query {
    id          = "ad1"
    return_data = true
    expression  = "ANOMALY_DETECTION_BAND(m1, 2)"
  }
}

# ---------------------------------------------------------------------------
# Outputs
# ---------------------------------------------------------------------------

output "cost_alert_topic_arn" {
  value = aws_sns_topic.cost_alerts.arn
}
