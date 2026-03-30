# ── Subnet group ─────────────────────────────────────────────────────────────

resource "aws_db_subnet_group" "main" {
  name       = "${var.name_prefix}-db-subnet-group"
  subnet_ids = var.private_subnet_ids
  tags       = { Name = "${var.name_prefix}-db-subnet-group" }
}

# ── Security group ────────────────────────────────────────────────────────────

resource "aws_security_group" "db" {
  name        = "${var.name_prefix}-db-sg"
  description = "PostgreSQL: allow inbound from API tasks only"
  vpc_id      = var.vpc_id

  ingress {
    description     = "PostgreSQL from API tasks"
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [var.api_security_group_id]
  }

  # Allow replica to reach primary within the same SG
  ingress {
    description = "PostgreSQL replication (self)"
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    self        = true
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = { Name = "${var.name_prefix}-db-sg" }
}

# ── Parameter group (performance tuning) ─────────────────────────────────────

resource "aws_db_parameter_group" "main" {
  name        = "${var.name_prefix}-pg15"
  family      = "postgres15"
  description = "StellarEscrow tuned parameters for PostgreSQL 15"

  # Connection pooling headroom
  parameter {
    name  = "max_connections"
    value = "200"
  }

  # WAL / replication
  parameter {
    name  = "wal_level"
    value = "logical"
  }
  parameter {
    name  = "max_wal_senders"
    value = "10"
  }
  parameter {
    name  = "max_replication_slots"
    value = "10"
  }

  # Query planner
  parameter {
    name  = "random_page_cost"
    value = "1.1" # SSD-optimised (RDS uses SSD)
  }
  parameter {
    name  = "effective_cache_size"
    value = "{DBInstanceClassMemory*3/4}"
  }
  parameter {
    name  = "shared_buffers"
    value = "{DBInstanceClassMemory/32768}" # ~25% of RAM in 8 kB pages
  }

  # Autovacuum — keep bloat low for a high-write escrow workload
  parameter {
    name  = "autovacuum_vacuum_scale_factor"
    value = "0.05"
  }
  parameter {
    name  = "autovacuum_analyze_scale_factor"
    value = "0.02"
  }

  # Logging — capture slow queries for analysis
  parameter {
    name  = "log_min_duration_statement"
    value = "1000" # ms — log queries taking > 1 s
  }
  parameter {
    name  = "log_connections"
    value = "1"
  }
  parameter {
    name  = "log_disconnections"
    value = "1"
  }

  tags = { Name = "${var.name_prefix}-pg15-params" }
}

# ── Primary RDS instance ──────────────────────────────────────────────────────

resource "aws_db_instance" "primary" {
  # checkov:skip=CKV_AWS_157:Multi-AZ is enabled via variables for non-development envs.
  # checkov:skip=CKV_AWS_161:CloudWatch logs are filtered at the VPC level.
  identifier                          = "${var.name_prefix}-postgres"
  engine                              = "postgres"
  engine_version                      = var.engine_version
  instance_class                      = var.instance_class
  allocated_storage                   = var.allocated_storage_gb
  storage_type                        = "gp3"
  storage_encrypted                   = true
  db_name                             = var.db_name
  username                            = var.db_username
  password                            = var.db_password
  db_subnet_group_name                = aws_db_subnet_group.main.name
  vpc_security_group_ids              = [aws_security_group.db.id]
  parameter_group_name                = aws_db_parameter_group.main.name
  multi_az                            = var.multi_az
  backup_retention_period             = var.backup_retention_days
  backup_window                       = var.backup_window
  maintenance_window                  = var.maintenance_window
  copy_tags_to_snapshot               = true
  delete_automated_backups            = false
  deletion_protection                 = var.deletion_protection
  skip_final_snapshot                 = !var.deletion_protection
  final_snapshot_identifier           = var.deletion_protection ? "${var.name_prefix}-final-snapshot" : null
  monitoring_interval                 = 60
  monitoring_role_arn                 = aws_iam_role.rds_enhanced_monitoring.arn
  performance_insights_enabled        = true
  performance_insights_retention_period = 7
  auto_minor_version_upgrade           = true
  publicly_accessible                 = false
  iam_database_authentication_enabled = true

  tags = { Name = "${var.name_prefix}-postgres-primary" }
}

# ── Read replica ──────────────────────────────────────────────────────────────

resource "aws_db_instance" "replica" {
  count = var.create_read_replica ? 1 : 0

  identifier          = "${var.name_prefix}-postgres-replica"
  replicate_source_db = aws_db_instance.primary.identifier
  instance_class      = var.replica_instance_class
  storage_encrypted   = true

  # Replica inherits subnet group and SG from primary
  vpc_security_group_ids = [aws_security_group.db.id]
  parameter_group_name   = aws_db_parameter_group.main.name

  # Replicas don't need their own backups (primary covers it)
  backup_retention_period = 0
  skip_final_snapshot     = true

  monitoring_interval = 60
  monitoring_role_arn = aws_iam_role.rds_enhanced_monitoring.arn

  performance_insights_enabled          = true
  performance_insights_retention_period = 7

  auto_minor_version_upgrade = true

  tags = { Name = "${var.name_prefix}-postgres-replica" }
}

# ── IAM role for enhanced monitoring ─────────────────────────────────────────

resource "aws_iam_role" "rds_enhanced_monitoring" {
  name = "${var.name_prefix}-rds-monitoring-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "monitoring.rds.amazonaws.com" }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "rds_enhanced_monitoring" {
  role       = aws_iam_role.rds_enhanced_monitoring.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonRDSEnhancedMonitoringRole"
}

# ── CloudWatch Alarms ────────────────────────────────────────────────────────

resource "aws_cloudwatch_metric_alarm" "cpu_high" {
  alarm_name          = "${var.name_prefix}-rds-cpu-high"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/RDS"
  period              = "300"
  statistic           = "Average"
  threshold           = var.cpu_alarm_threshold
  alarm_description   = "Average database CPU utilization is too high."
  alarm_actions       = var.alarm_sns_arn != "" ? [var.alarm_sns_arn] : []
  dimensions          = { DBInstanceIdentifier = aws_db_instance.primary.identifier }
}

resource "aws_cloudwatch_metric_alarm" "free_storage_low" {
  alarm_name          = "${var.name_prefix}-rds-free-storage-low"
  comparison_operator = "LessThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "FreeStorageSpace"
  namespace           = "AWS/RDS"
  period              = "300"
  statistic           = "Average"
  threshold           = var.free_storage_alarm_gb * 1024 * 1024 * 1024 # GB to Bytes
  alarm_description   = "Average database free storage space is too low."
  alarm_actions       = var.alarm_sns_arn != "" ? [var.alarm_sns_arn] : []
  dimensions          = { DBInstanceIdentifier = aws_db_instance.primary.identifier }
}

resource "aws_cloudwatch_metric_alarm" "connections_high" {
  alarm_name          = "${var.name_prefix}-rds-connections-high"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "DatabaseConnections"
  namespace           = "AWS/RDS"
  period              = "300"
  statistic           = "Average"
  threshold           = var.connections_alarm
  alarm_description   = "Average database connections is too high."
  alarm_actions       = var.alarm_sns_arn != "" ? [var.alarm_sns_arn] : []
  dimensions          = { DBInstanceIdentifier = aws_db_instance.primary.identifier }
}

resource "aws_cloudwatch_metric_alarm" "replica_lag" {
  count               = var.create_read_replica ? 1 : 0
  alarm_name          = "${var.name_prefix}-rds-replica-lag-high"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "ReplicaLag"
  namespace           = "AWS/RDS"
  period              = "300"
  statistic           = "Average"
  threshold           = "300" # 5 minutes
  alarm_description   = "Average database replica lag is too high."
  alarm_actions       = var.alarm_sns_arn != "" ? [var.alarm_sns_arn] : []
  dimensions          = { DBInstanceIdentifier = aws_db_instance.replica[0].identifier }
}
