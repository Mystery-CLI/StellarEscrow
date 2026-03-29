environment        = "staging"
aws_region         = "us-east-1"
app_version        = "latest"
vpc_cidr           = "10.1.0.0/16"
availability_zones = ["us-east-1a", "us-east-1b"]

db_instance_class           = "db.t3.small"
db_name                     = "stellar_escrow"
db_username                 = "indexer"
db_allocated_storage_gb     = 20
db_max_allocated_storage_gb = 100
db_engine_version           = "15.6"
db_backup_window            = "03:00-04:00"
db_maintenance_window       = "Mon:04:00-Mon:05:00"

api_image          = "stellarescrow/api:staging"
api_cpu            = 512
api_memory         = 1024
api_container_port = 3000

certificate_arn = ""
alarm_sns_arn   = ""

stellar_network     = "testnet"
stellar_horizon_url = "https://horizon-testnet.stellar.org"
stellar_contract_id = ""
