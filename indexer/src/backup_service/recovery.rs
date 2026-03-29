use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub generated_at: DateTime<Utc>,
    pub steps: Vec<RecoveryStep>,
    pub estimated_rto_minutes: u32,
    pub estimated_rpo_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStep {
    pub order: u32,
    pub action: String,
    pub command: Option<String>,
    pub notes: String,
}

/// Generate a recovery plan based on the latest available backup.
pub fn generate_recovery_plan(backup_location: Option<&str>) -> RecoveryPlan {
    let backup_source = backup_location.unwrap_or("<latest-backup>");

    RecoveryPlan {
        generated_at: Utc::now(),
        estimated_rto_minutes: 30,
        estimated_rpo_minutes: 60,
        steps: vec![
            RecoveryStep {
                order: 1,
                action: "Stop all services".to_string(),
                command: Some("docker-compose down".to_string()),
                notes: "Ensure no writes occur during recovery".to_string(),
            },
            RecoveryStep {
                order: 2,
                action: "Download backup".to_string(),
                command: Some(format!("aws s3 cp {} /tmp/recovery.sql.gz", backup_source)),
                notes: "Skip if backup is already local".to_string(),
            },
            RecoveryStep {
                order: 3,
                action: "Restore database".to_string(),
                command: Some("gunzip -c /tmp/recovery.sql.gz | psql $DATABASE_URL".to_string()),
                notes: "Ensure DATABASE_URL points to the target database".to_string(),
            },
            RecoveryStep {
                order: 4,
                action: "Run migrations".to_string(),
                command: Some("sqlx migrate run".to_string()),
                notes: "Apply any migrations that post-date the backup".to_string(),
            },
            RecoveryStep {
                order: 5,
                action: "Restart services".to_string(),
                command: Some("docker-compose up -d".to_string()),
                notes: "Verify health endpoints after restart".to_string(),
            },
            RecoveryStep {
                order: 6,
                action: "Verify recovery".to_string(),
                command: Some("curl http://localhost:3000/health".to_string()),
                notes: "Check /health, /status, and /stats endpoints".to_string(),
            },
        ],
    }
}
