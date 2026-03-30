pub mod types;
pub mod worker;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::error::AppError;
use types::{Job, JobMonitorSnapshot, JobPriority, QueueStats};

pub const QUEUE_CRITICAL: &str = "stellar_escrow:jobs:critical";
pub const QUEUE_HIGH: &str = "stellar_escrow:jobs:high";
pub const QUEUE_NORMAL: &str = "stellar_escrow:jobs:normal";
pub const QUEUE_LOW: &str = "stellar_escrow:jobs:low";
pub const QUEUE_SCHEDULED: &str = "stellar_escrow:jobs:scheduled";
pub const QUEUE_DEAD_LETTER: &str = "stellar_escrow:jobs:dead_letter";
pub const QUEUE_RETRIES: &str = "stellar_escrow:jobs:retries";
pub const QUEUE_PROCESSED: &str = "stellar_escrow:jobs:processed";

#[derive(Clone)]
pub struct JobQueue {
    conn: Option<ConnectionManager>,
}

impl JobQueue {
    pub async fn new(redis_url: &str) -> Result<Self, AppError> {
        if redis_url.trim().is_empty() {
            tracing::warn!("Job queue disabled: cache.redis_url is empty");
            return Ok(Self { conn: None });
        }

        let client = redis::Client::open(redis_url)
            .map_err(|e| AppError::Storage(format!("Redis client error: {}", e)))?;
        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::Storage(format!("Redis connection error: {}", e)))?;
        Ok(Self { conn: Some(conn) })
    }

    pub async fn enqueue(&mut self, job: Job) -> Result<(), AppError> {
        if let Some(run_at) = job.run_at {
            return self.enqueue_at(job, run_at).await;
        }

        let queue = queue_for_priority(&job.priority);
        let json = serde_json::to_string(&job)?;
        let Some(conn) = self.conn.as_mut() else {
            return Ok(());
        };
        conn
            .rpush::<_, _, ()>(queue, json)
            .await
            .map_err(|e| AppError::Storage(format!("Redis enqueue error: {}", e)))?;
        Ok(())
    }

    pub async fn enqueue_at(&mut self, job: Job, run_at: i64) -> Result<(), AppError> {
        let json = serde_json::to_string(&job)?;
        let Some(conn) = self.conn.as_mut() else {
            return Ok(());
        };
        conn
            .zadd::<_, _, _, ()>(QUEUE_SCHEDULED, json, run_at)
            .await
            .map_err(|e| AppError::Storage(format!("Redis schedule error: {}", e)))?;
        Ok(())
    }

    pub async fn record_retry(&mut self, job: &Job) -> Result<(), AppError> {
        let json = serde_json::to_string(job)?;
        let Some(conn) = self.conn.as_mut() else {
            return Ok(());
        };
        conn
            .lpush::<_, _, ()>(QUEUE_RETRIES, json)
            .await
            .map_err(|e| AppError::Storage(format!("Redis retry record error: {}", e)))?;
        Ok(())
    }

    pub async fn record_processed(&mut self, job: &Job) -> Result<(), AppError> {
        let json = serde_json::to_string(job)?;
        let Some(conn) = self.conn.as_mut() else {
            return Ok(());
        };
        conn
            .lpush::<_, _, ()>(QUEUE_PROCESSED, json)
            .await
            .map_err(|e| AppError::Storage(format!("Redis processed record error: {}", e)))?;
        conn
            .ltrim::<_, ()>(QUEUE_PROCESSED, 0, 199)
            .await
            .map_err(|e| AppError::Storage(format!("Redis processed trim error: {}", e)))?;
        Ok(())
    }

    pub async fn dead_letter(&mut self, job: &Job) -> Result<(), AppError> {
        let json = serde_json::to_string(job)?;
        let Some(conn) = self.conn.as_mut() else {
            return Ok(());
        };
        conn
            .lpush::<_, _, ()>(QUEUE_DEAD_LETTER, json)
            .await
            .map_err(|e| AppError::Storage(format!("Redis dead-letter error: {}", e)))?;
        Ok(())
    }

    pub async fn dequeue(&mut self) -> Result<Option<Job>, AppError> {
        for queue in [QUEUE_CRITICAL, QUEUE_HIGH, QUEUE_NORMAL, QUEUE_LOW] {
            let Some(conn) = self.conn.as_mut() else {
                return Ok(None);
            };
            let payload: Option<String> = conn
                .lpop(queue, None)
                .await
                .map_err(|e| AppError::Storage(format!("Redis pop error: {}", e)))?;
            if let Some(payload) = payload {
                let job = serde_json::from_str(&payload)?;
                return Ok(Some(job));
            }
        }
        Ok(None)
    }

    pub async fn promote_scheduled(&mut self, now: i64) -> Result<u64, AppError> {
        let Some(conn) = self.conn.as_mut() else {
            return Ok(0);
        };
        let due: Vec<String> = conn
            .zrangebyscore(QUEUE_SCHEDULED, "-inf", now)
            .await
            .map_err(|e| AppError::Storage(format!("Redis schedule fetch error: {}", e)))?;

        let mut promoted = 0u64;
        for json in due {
            conn
                .zrem::<_, _, ()>(QUEUE_SCHEDULED, &json)
                .await
                .map_err(|e| AppError::Storage(format!("Redis zrem error: {}", e)))?;

            let job: Job = serde_json::from_str(&json)?;
            let queue = queue_for_priority(&job.priority);
            conn
                .rpush::<_, _, ()>(queue, json)
                .await
                .map_err(|e| AppError::Storage(format!("Redis promote error: {}", e)))?;
            promoted += 1;
        }

        Ok(promoted)
    }

    pub async fn stats(&mut self) -> Result<QueueStats, AppError> {
        Ok(QueueStats {
            critical: self.len(QUEUE_CRITICAL).await?,
            high: self.len(QUEUE_HIGH).await?,
            normal: self.len(QUEUE_NORMAL).await?,
            low: self.len(QUEUE_LOW).await?,
            scheduled: if let Some(conn) = self.conn.as_mut() {
                conn.zcard(QUEUE_SCHEDULED)
                    .await
                    .map_err(|e| AppError::Storage(format!("Redis zcard error: {}", e)))?
            } else {
                0
            },
            dead_letter: self.len(QUEUE_DEAD_LETTER).await?,
            retries: self.len(QUEUE_RETRIES).await?,
            processed: self.len(QUEUE_PROCESSED).await?,
        })
    }

    pub async fn snapshot(&mut self, worker_name: &str, running: bool) -> Result<JobMonitorSnapshot, AppError> {
        Ok(JobMonitorSnapshot {
            stats: self.stats().await?,
            running,
            worker_name: worker_name.to_string(),
        })
    }

    async fn len(&mut self, queue: &str) -> Result<i64, AppError> {
        let Some(conn) = self.conn.as_mut() else {
            return Ok(0);
        };
        conn.llen(queue)
            .await
            .map_err(|e| AppError::Storage(format!("Redis llen error: {}", e)))
    }
}

pub fn queue_for_priority(priority: &JobPriority) -> &'static str {
    match priority {
        JobPriority::Critical => QUEUE_CRITICAL,
        JobPriority::High => QUEUE_HIGH,
        JobPriority::Normal => QUEUE_NORMAL,
        JobPriority::Low => QUEUE_LOW,
    }
}

#[cfg(test)]
mod tests {
    use super::{queue_for_priority, QUEUE_CRITICAL, QUEUE_HIGH, QUEUE_LOW, QUEUE_NORMAL};
    use crate::job_queue::types::JobPriority;

    #[test]
    fn maps_priorities_to_expected_queues() {
        assert_eq!(queue_for_priority(&JobPriority::Critical), QUEUE_CRITICAL);
        assert_eq!(queue_for_priority(&JobPriority::High), QUEUE_HIGH);
        assert_eq!(queue_for_priority(&JobPriority::Normal), QUEUE_NORMAL);
        assert_eq!(queue_for_priority(&JobPriority::Low), QUEUE_LOW);
    }
}
