environment        = "production"
aws_region         = "us-east-1"
app_version        = "latest"
vpc_cidr           = "10.2.0.0/16"
availability_zones = ["us-east-1a", "us-east-1b"]

db_instance_class           = "db.t3.medium"
db_name                     = "stellar_escrow"
db_username                 = "indexer"
db_allocated_storage_gb     = 50
db_max_allocated_storage_gb = 500
db_engine_version           = "15.6"
db_backup_window            = "03:00-04:00"
db_maintenance_window       = "Mon:04:00-Mon:05:00"

api_image          = "stellarescrow/api:production"
api_cpu            = 1024
api_memory         = 2048
api_container_port = 3000

# Inject via TF_VAR_certificate_arn and TF_VAR_alarm_sns_arn in CI
certificate_arn = ""
alarm_sns_arn   = ""

stellar_network     = "mainnet"
stellar_horizon_url = "https://horizon.stellar.org"
stellar_contract_id = ""
# db_password — inject via TF_VAR_db_password (never commit)
