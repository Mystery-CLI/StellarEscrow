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
  identifier        = "${var.name_prefix}-postgres"
  engine            = "postgres"
  engine_version    = var.engine_version
  instance_class    = var.instance_class
  allocated_storage = var.allocated_storage_gb
  storage_type      = "gp3"
  storage_encrypted = true
  iops              = 0 # gp3 baseline; set > 0 to provision IOPS

  # Storage autoscaling — grows automatically up to max_allocated_storage_gb
  max_allocated_storage = var.max_allocated_storage_gb > 0 ? var.max_allocated_storage_gb : null

  db_name  = var.db_name
  username = var.db_username
  password = var.db_password

  db_subnet_group_name   = aws_db_subnet_group.main.name
  vpc_security_group_ids = [aws_security_group.db.id]
  parameter_group_name   = aws_db_parameter_group.main.name

  # High availability
  multi_az = var.multi_az

  # Automated backups
  backup_retention_period  = var.backup_retention_days
  backup_window            = var.backup_window
  maintenance_window       = var.maintenance_window
  copy_tags_to_snapshot    = true
  delete_automated_backups = false

  # Final snapshot on destroy (skipped only when deletion_protection is off)
  deletion_protection       = var.deletion_protection
  skip_final_snapshot       = !var.deletion_protection
  final_snapshot_identifier = var.deletion_protection ? "${var.name_prefix}-final-snapshot" : null

  # Enhanced monitoring — 60-second granularity
  monitoring_interval = 60
  monitoring_role_arn = aws_iam_role.rds_enhanced_monitoring.arn

  # Performance Insights — 7-day free retention
  performance_insights_enabled          = true
  performance_insights_retention_period = 7

  # Auto minor version upgrades during maintenance window
  auto_minor_version_upgrade = true

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
