pub mod types;
pub mod worker;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::error::AppError;
use types::{Job, JobPriority};

const QUEUE_HIGH: &str = "stellar_escrow:jobs:high";
const QUEUE_NORMAL: &str = "stellar_escrow:jobs:normal";
const QUEUE_SCHEDULED: &str = "stellar_escrow:jobs:scheduled";

#[derive(Debug)]
pub struct QueueStats {
    pub high_priority: i64,
    pub normal: i64,
    pub scheduled: i64,
}

pub struct JobQueue {
    conn: ConnectionManager,
}

impl JobQueue {
    pub async fn new(redis_url: &str) -> Result<Self, AppError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AppError::Internal(format!("Redis client error: {}", e)))?;
        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::Internal(format!("Redis connection error: {}", e)))?;
        Ok(Self { conn })
    }

    pub async fn enqueue(&mut self, job: Job) -> Result<(), AppError> {
        let queue = match job.priority {
            JobPriority::High => QUEUE_HIGH,
            JobPriority::Normal => QUEUE_NORMAL,
        };
        let json = serde_json::to_string(&job)
            .map_err(|e| AppError::Internal(format!("Job serialization error: {}", e)))?;
        self.conn
            .rpush::<_, _, ()>(queue, json)
            .await
            .map_err(|e| AppError::Internal(format!("Redis enqueue error: {}", e)))?;
        Ok(())
    }

    /// Schedule a job to run at a specific Unix timestamp.
    pub async fn enqueue_at(&mut self, job: Job, run_at: i64) -> Result<(), AppError> {
        let json = serde_json::to_string(&job)
            .map_err(|e| AppError::Internal(format!("Job serialization error: {}", e)))?;
        self.conn
            .zadd::<_, _, _, ()>(QUEUE_SCHEDULED, json, run_at)
            .await
            .map_err(|e| AppError::Internal(format!("Redis schedule error: {}", e)))?;
        Ok(())
    }

    /// Returns current queue depths for monitoring.
    pub async fn stats(&mut self) -> Result<QueueStats, AppError> {
        let high_priority = self.conn
            .llen(QUEUE_HIGH)
            .await
            .map_err(|e| AppError::Internal(format!("Redis llen error: {}", e)))?;
        let normal = self.conn
            .llen(QUEUE_NORMAL)
            .await
            .map_err(|e| AppError::Internal(format!("Redis llen error: {}", e)))?;
        let scheduled = self.conn
            .zcard(QUEUE_SCHEDULED)
            .await
            .map_err(|e| AppError::Internal(format!("Redis zcard error: {}", e)))?;
        Ok(QueueStats { high_priority, normal, scheduled })
    }
}
