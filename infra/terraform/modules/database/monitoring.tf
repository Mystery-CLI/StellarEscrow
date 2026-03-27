locals {
  alarm_actions = var.alarm_sns_arn != "" ? [var.alarm_sns_arn] : []
}

# ── CloudWatch Alarms ─────────────────────────────────────────────────────────

resource "aws_cloudwatch_metric_alarm" "cpu_high" {
  alarm_name          = "${var.name_prefix}-db-cpu-high"
  alarm_description   = "RDS CPU utilisation above ${var.cpu_alarm_threshold}% for 5 minutes"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 5
  threshold           = var.cpu_alarm_threshold
  treat_missing_data  = "notBreaching"
  namespace           = "AWS/RDS"
  metric_name         = "CPUUtilization"
  dimensions          = { DBInstanceIdentifier = aws_db_instance.primary.identifier }
  period              = 60
  statistic           = "Average"
  alarm_actions       = local.alarm_actions
  ok_actions          = local.alarm_actions
}

resource "aws_cloudwatch_metric_alarm" "free_storage_low" {
  alarm_name          = "${var.name_prefix}-db-storage-low"
  alarm_description   = "RDS free storage below ${var.free_storage_alarm_gb} GB"
  comparison_operator = "LessThanThreshold"
  evaluation_periods  = 3
  threshold           = var.free_storage_alarm_gb * 1073741824 # GB → bytes
  treat_missing_data  = "notBreaching"
  namespace           = "AWS/RDS"
  metric_name         = "FreeStorageSpace"
  dimensions          = { DBInstanceIdentifier = aws_db_instance.primary.identifier }
  period              = 300
  statistic           = "Average"
  alarm_actions       = local.alarm_actions
  ok_actions          = local.alarm_actions
}

resource "aws_cloudwatch_metric_alarm" "connections_high" {
  alarm_name          = "${var.name_prefix}-db-connections-high"
  alarm_description   = "RDS connection count above ${var.connections_alarm}"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 3
  threshold           = var.connections_alarm
  treat_missing_data  = "notBreaching"
  namespace           = "AWS/RDS"
  metric_name         = "DatabaseConnections"
  dimensions          = { DBInstanceIdentifier = aws_db_instance.primary.identifier }
  period              = 60
  statistic           = "Average"
  alarm_actions       = local.alarm_actions
  ok_actions          = local.alarm_actions
}

resource "aws_cloudwatch_metric_alarm" "read_latency_high" {
  alarm_name          = "${var.name_prefix}-db-read-latency"
  alarm_description   = "RDS read latency above 20ms for 5 minutes"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 5
  threshold           = 0.02 # seconds
  treat_missing_data  = "notBreaching"
  namespace           = "AWS/RDS"
  metric_name         = "ReadLatency"
  dimensions          = { DBInstanceIdentifier = aws_db_instance.primary.identifier }
  period              = 60
  statistic           = "Average"
  alarm_actions       = local.alarm_actions
  ok_actions          = local.alarm_actions
}

resource "aws_cloudwatch_metric_alarm" "write_latency_high" {
  alarm_name          = "${var.name_prefix}-db-write-latency"
  alarm_description   = "RDS write latency above 20ms for 5 minutes"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 5
  threshold           = 0.02
  treat_missing_data  = "notBreaching"
  namespace           = "AWS/RDS"
  metric_name         = "WriteLatency"
  dimensions          = { DBInstanceIdentifier = aws_db_instance.primary.identifier }
  period              = 60
  statistic           = "Average"
  alarm_actions       = local.alarm_actions
  ok_actions          = local.alarm_actions
}

# Replica lag alarm — only created when a replica exists
resource "aws_cloudwatch_metric_alarm" "replica_lag" {
  count = var.create_read_replica ? 1 : 0

  alarm_name          = "${var.name_prefix}-db-replica-lag"
  alarm_description   = "RDS replica lag above 30 seconds"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 3
  threshold           = 30
  treat_missing_data  = "notBreaching"
  namespace           = "AWS/RDS"
  metric_name         = "ReplicaLag"
  dimensions          = { DBInstanceIdentifier = aws_db_instance.replica[0].identifier }
  period              = 60
  statistic           = "Average"
  alarm_actions       = local.alarm_actions
  ok_actions          = local.alarm_actions
}

# ── CloudWatch Dashboard ──────────────────────────────────────────────────────

resource "aws_cloudwatch_dashboard" "db" {
  dashboard_name = "${var.name_prefix}-database"

  dashboard_body = jsonencode({
    widgets = [
      {
        type   = "metric"
        x      = 0
        y      = 0
        width  = 12
        height = 6
        properties = {
          title  = "CPU Utilisation"
          region = "us-east-1"
          metrics = [
            ["AWS/RDS", "CPUUtilization",
              "DBInstanceIdentifier", aws_db_instance.primary.identifier,
            { stat = "Average", period = 60, label = "Primary CPU %" }]
          ]
          annotations = { horizontal = [{ value = var.cpu_alarm_threshold, label = "Alarm threshold", color = "#d62728" }] }
          view        = "timeSeries"
        }
      },
      {
        type   = "metric"
        x      = 12
        y      = 0
        width  = 12
        height = 6
        properties = {
          title  = "Database Connections"
          region = "us-east-1"
          metrics = [
            ["AWS/RDS", "DatabaseConnections",
              "DBInstanceIdentifier", aws_db_instance.primary.identifier,
            { stat = "Average", period = 60, label = "Connections" }]
          ]
          annotations = { horizontal = [{ value = var.connections_alarm, label = "Alarm threshold", color = "#d62728" }] }
          view        = "timeSeries"
        }
      },
      {
        type   = "metric"
        x      = 0
        y      = 6
        width  = 12
        height = 6
        properties = {
          title  = "Read / Write Latency"
          region = "us-east-1"
          metrics = [
            ["AWS/RDS", "ReadLatency", "DBInstanceIdentifier", aws_db_instance.primary.identifier, { stat = "Average", period = 60, label = "Read (s)" }],
            ["AWS/RDS", "WriteLatency", "DBInstanceIdentifier", aws_db_instance.primary.identifier, { stat = "Average", period = 60, label = "Write (s)" }]
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
          title  = "Free Storage Space"
          region = "us-east-1"
          metrics = [
            ["AWS/RDS", "FreeStorageSpace",
              "DBInstanceIdentifier", aws_db_instance.primary.identifier,
            { stat = "Average", period = 300, label = "Free Storage (bytes)" }]
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
          title  = "IOPS"
          region = "us-east-1"
          metrics = [
            ["AWS/RDS", "ReadIOPS", "DBInstanceIdentifier", aws_db_instance.primary.identifier, { stat = "Average", period = 60, label = "Read IOPS" }],
            ["AWS/RDS", "WriteIOPS", "DBInstanceIdentifier", aws_db_instance.primary.identifier, { stat = "Average", period = 60, label = "Write IOPS" }]
          ]
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
          title  = "Freeable Memory"
          region = "us-east-1"
          metrics = [
            ["AWS/RDS", "FreeableMemory",
              "DBInstanceIdentifier", aws_db_instance.primary.identifier,
            { stat = "Average", period = 60, label = "Freeable Memory (bytes)" }]
          ]
          view = "timeSeries"
        }
      }
    ]
  })
}
