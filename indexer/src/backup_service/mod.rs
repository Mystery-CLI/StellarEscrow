pub mod verify;
pub mod recovery;

use crate::config::BackupConfig;
use crate::database::Database;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BackupStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Verified,
    VerificationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: BackupStatus,
    pub size_bytes: Option<u64>,
    pub location: Option<String>,
    pub checksum: Option<String>,
    pub error: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMonitorSnapshot {
    pub last_backup: Option<BackupRecord>,
    pub last_successful_at: Option<DateTime<Utc>>,
    pub total_backups: u64,
    pub failed_backups: u64,
    pub next_scheduled: Option<DateTime<Utc>>,
    pub storage_used_bytes: u64,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct BackupService {
    db: Arc<Database>,
    config: BackupConfig,
    history: Arc<RwLock<Vec<BackupRecord>>>,
}

impl BackupService {
    pub fn new(db: Arc<Database>, config: BackupConfig) -> Self {
        Self {
            db,
            config,
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Trigger a backup immediately (called by scheduler or admin endpoint).
    pub async fn run_backup(&self) -> BackupRecord {
        let record = BackupRecord {
            id: Uuid::new_v4(),
            started_at: Utc::now(),
            completed_at: None,
            status: BackupStatus::Running,
            size_bytes: None,
            location: None,
            checksum: None,
            error: None,
            verified: false,
        };

        self.history.write().await.push(record.clone());

        // Persist to DB
        if let Err(e) = self.db.insert_backup_record(&record).await {
            tracing::error!("Failed to persist backup record: {}", e);
        }

        // Trigger the backup shell script asynchronously
        let config = self.config.clone();
        let record_id = record.id;
        let db = self.db.clone();
        let history = self.history.clone();

        tokio::spawn(async move {
            let result = run_backup_script(&config).await;
            let mut completed = BackupRecord {
                id: record_id,
                started_at: record.started_at,
                completed_at: Some(Utc::now()),
                status: if result.is_ok() { BackupStatus::Completed } else { BackupStatus::Failed },
                size_bytes: result.as_ref().ok().and_then(|r| r.size_bytes),
                location: result.as_ref().ok().and_then(|r| r.location.clone()),
                checksum: result.as_ref().ok().and_then(|r| r.checksum.clone()),
                error: result.err().map(|e| e.to_string()),
                verified: false,
            };

            // Auto-verify if completed
            if completed.status == BackupStatus::Completed {
                if let Some(ref loc) = completed.location {
                    completed.verified = verify::verify_backup(loc, completed.checksum.as_deref()).await;
                    completed.status = if completed.verified {
                        BackupStatus::Verified
                    } else {
                        BackupStatus::VerificationFailed
                    };
                }
            }

            // Update history
            let mut h = history.write().await;
            if let Some(r) = h.iter_mut().find(|r| r.id == record_id) {
                *r = completed.clone();
            }

            // Persist final state
            if let Err(e) = db.update_backup_record(&completed).await {
                tracing::error!("Failed to update backup record: {}", e);
            }

            if completed.status == BackupStatus::Failed || completed.status == BackupStatus::VerificationFailed {
                tracing::error!("Backup {} failed: {:?}", record_id, completed.error);
            } else {
                tracing::info!("Backup {} completed and verified", record_id);
            }
        });

        record
    }

    /// Get monitoring snapshot.
    pub async fn get_monitor_snapshot(&self) -> BackupMonitorSnapshot {
        let history = self.history.read().await;
        let last_backup = history.last().cloned();
        let last_successful_at = history.iter()
            .filter(|r| r.status == BackupStatus::Verified || r.status == BackupStatus::Completed)
            .last()
            .and_then(|r| r.completed_at);
        let failed = history.iter().filter(|r| r.status == BackupStatus::Failed).count() as u64;
        let storage_used: u64 = history.iter().filter_map(|r| r.size_bytes).sum();

        BackupMonitorSnapshot {
            last_backup,
            last_successful_at,
            total_backups: history.len() as u64,
            failed_backups: failed,
            next_scheduled: next_scheduled_time(&self.config),
            storage_used_bytes: storage_used,
        }
    }

    /// List backup history.
    pub async fn list_backups(&self, limit: usize) -> Vec<BackupRecord> {
        let history = self.history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Background scheduler — runs backups on the configured interval.
    pub async fn run_scheduler(self: Arc<Self>) {
        let interval_secs = self.config.interval_hours * 3600;
        let duration = std::time::Duration::from_secs(interval_secs);
        loop {
            tokio::time::sleep(duration).await;
            tracing::info!("Scheduled backup starting");
            self.run_backup().await;
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct BackupResult {
    size_bytes: Option<u64>,
    location: Option<String>,
    checksum: Option<String>,
}

async fn run_backup_script(config: &BackupConfig) -> anyhow::Result<BackupResult> {
    let script = config.script_path.as_deref().unwrap_or("scripts/backup.sh");

    let output = tokio::process::Command::new("bash")
        .arg(script)
        .env("DATABASE_URL", &config.database_url)
        .env("BACKUP_DIR", config.backup_dir.as_deref().unwrap_or("/var/backups/stellarescrow"))
        .env("S3_BUCKET", config.s3_bucket.as_deref().unwrap_or(""))
        .env("RETENTION_DAYS", config.retention_days.to_string())
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Backup script failed: {}", stderr);
    }

    // Parse location from stdout (script prints "BACKUP_LOCATION=<path>")
    let stdout = String::from_utf8_lossy(&output.stdout);
    let location = stdout.lines()
        .find(|l| l.starts_with("BACKUP_LOCATION="))
        .map(|l| l.trim_start_matches("BACKUP_LOCATION=").to_string());

    Ok(BackupResult {
        size_bytes: None,
        location,
        checksum: None,
    })
}

fn next_scheduled_time(config: &BackupConfig) -> Option<DateTime<Utc>> {
    if config.interval_hours == 0 {
        return None;
    }
    Some(Utc::now() + chrono::Duration::hours(config.interval_hours as i64))
}
