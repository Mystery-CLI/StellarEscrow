use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use tracing::{error, info, warn};

use crate::error::AppError;
use super::types::Job;

const QUEUE_HIGH: &str = "stellar_escrow:jobs:high";
const QUEUE_NORMAL: &str = "stellar_escrow:jobs:normal";
const QUEUE_SCHEDULED: &str = "stellar_escrow:jobs:scheduled";
const MAX_ATTEMPTS: u32 = 3;

pub struct JobWorker {
    conn: ConnectionManager,
}

impl JobWorker {
    pub async fn new(redis_url: &str) -> Result<Self, AppError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AppError::Internal(format!("Redis client error: {}", e)))?;
        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::Internal(format!("Redis connection error: {}", e)))?;
        Ok(Self { conn })
    }

    pub async fn run(&mut self) -> Result<(), AppError> {
        let mut iterations: u64 = 0;

        loop {
            // Log queue stats every 60 iterations (~60 seconds at 1s sleep)
            if iterations % 60 == 0 {
                self.log_stats().await;
            }
            iterations += 1;

            // Move any due scheduled jobs into the normal queue
            self.promote_scheduled().await?;

            // Check high priority queue first, then normal
            let payload: Option<String> = self.conn
                .lpop(QUEUE_HIGH, None)
                .await
                .map_err(|e| AppError::Internal(format!("Redis pop error: {}", e)))?;

            let payload = match payload {
                Some(p) => Some(p),
                None => self.conn
                    .lpop(QUEUE_NORMAL, None)
                    .await
                    .map_err(|e| AppError::Internal(format!("Redis pop error: {}", e)))?,
            };

            if payload.is_none() {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }

            let job: Job = serde_json::from_str(&payload.unwrap())
                .map_err(|e| AppError::Internal(format!("Job deserialization error: {}", e)))?;

            self.process_with_retry(job).await;
        }
    }

    async fn process_with_retry(&self, job: Job) {
        for attempt in 1..=MAX_ATTEMPTS {
            match self.process(&job).await {
                Ok(_) => {
                    info!("[job-worker] job {} completed on attempt {}", job.event_id, attempt);
                    return;
                }
                Err(e) if attempt < MAX_ATTEMPTS => {
                    let delay = std::time::Duration::from_secs(2u64.pow((attempt - 1) as u32));
                    warn!("[job-worker] job {} failed (attempt {}/{}): {}. Retrying in {:?}",
                        job.event_id, attempt, MAX_ATTEMPTS, e, delay);
                    tokio::time::sleep(delay).await;
                }
                Err(e) => {
                    error!("[job-worker] job {} failed after {} attempts: {}",
                        job.event_id, MAX_ATTEMPTS, e);
                }
            }
        }
    }

    async fn process(&self, job: &Job) -> Result<(), AppError> {
        // TODO: replace with real service calls
        info!("[job-worker] processing job: {:?}", job);
        Ok(())
    }

    /// Logs current queue depths for observability.
    async fn log_stats(&mut self) {
        let high: i64 = self.conn.llen(QUEUE_HIGH).await.unwrap_or(-1);
        let normal: i64 = self.conn.llen(QUEUE_NORMAL).await.unwrap_or(-1);
        let scheduled: i64 = self.conn.zcard(QUEUE_SCHEDULED).await.unwrap_or(-1);
        info!("[job-worker] queue stats — high: {}, normal: {}, scheduled: {}",
            high, normal, scheduled);
    }

    /// Moves all scheduled jobs whose run_at <= now into the normal queue.
    async fn promote_scheduled(&mut self) -> Result<(), AppError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Fetch all jobs with score <= now
        let due: Vec<String> = self.conn
            .zrangebyscore(QUEUE_SCHEDULED, "-inf", now)
            .await
            .map_err(|e| AppError::Internal(format!("Redis schedule fetch error: {}", e)))?;

        for json in due {
            // Remove from scheduled set and push to normal queue
            self.conn
                .zrem::<_, _, ()>(QUEUE_SCHEDULED, &json)
                .await
                .map_err(|e| AppError::Internal(format!("Redis zrem error: {}", e)))?;
            self.conn
                .rpush::<_, _, ()>(QUEUE_NORMAL, &json)
                .await
                .map_err(|e| AppError::Internal(format!("Redis promote error: {}", e)))?;
            info!("[job-worker] promoted scheduled job to normal queue");
        }

        Ok(())
    }
}