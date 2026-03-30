locals {
  infra_version = "1.2.0" # bumped: production database module added

  name_prefix = "stellarescrow-${var.environment}"

  # ── Environment Configurations ──────────────────────────────────────────────

  env_config = {
    development = {
      db_multi_az                = false
      db_deletion_protected      = false
      db_backup_retention_days   = 1
      db_create_replica          = false
      db_replica_class           = "db.t3.micro"
      api_desired_count          = 1
      enable_deletion_protection = false
      enable_stickiness          = false
      autoscaling_min            = 1
      autoscaling_max            = 2
      deletion_protection        = false
    }
    staging = {
      db_multi_az                = false
      db_deletion_protected      = false
      db_backup_retention_days   = 3
      db_create_replica          = false
      db_replica_class           = "db.t3.micro"
      api_desired_count          = 1
      enable_deletion_protection = false
      enable_stickiness          = true
      autoscaling_min            = 1
      autoscaling_max            = 3
      deletion_protection        = false
    }
    production = {
      db_multi_az                = true
      db_deletion_protected      = true
      db_backup_retention_days   = 14
      db_create_replica          = true
      db_replica_class           = "db.t3.medium"
      api_desired_count          = 2
      enable_deletion_protection = true
      enable_stickiness          = true
      autoscaling_min            = 2
      autoscaling_max            = 8
      deletion_protection        = true
    }
  }

  cfg = local.env_config[var.environment]
}
