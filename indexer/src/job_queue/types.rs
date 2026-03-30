use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    Event,
    Notification,
    CacheWarm,
    Compliance,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum JobPriority {
    Critical,
    High,
    #[default]
    Normal,
    Low,
}

impl JobPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobPriority::Critical => "critical",
            JobPriority::High => "high",
            JobPriority::Normal => "normal",
            JobPriority::Low => "low",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub event_id: String,
    pub trade_id: String,
    pub payload: Value,
    #[serde(default)]
    pub priority: JobPriority,
    #[serde(default)]
    pub attempts: u32,
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_at: Option<i64>,
    #[serde(default = "default_created_at")]
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

impl Job {
    pub fn new(
        job_type: JobType,
        event_id: impl Into<String>,
        trade_id: impl Into<String>,
        payload: Value,
        priority: JobPriority,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            job_type,
            event_id: event_id.into(),
            trade_id: trade_id.into(),
            payload,
            priority,
            attempts: 0,
            max_attempts: default_max_attempts(),
            run_at: None,
            created_at: default_created_at(),
            last_error: None,
        }
    }

    pub fn scheduled_at(mut self, run_at: i64) -> Self {
        self.run_at = Some(run_at);
        self
    }

    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueueStats {
    pub critical: i64,
    pub high: i64,
    pub normal: i64,
    pub low: i64,
    pub scheduled: i64,
    pub dead_letter: i64,
    pub retries: i64,
    pub processed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMonitorSnapshot {
    pub stats: QueueStats,
    pub running: bool,
    pub worker_name: String,
}

fn default_max_attempts() -> u32 {
    3
}

fn default_created_at() -> i64 {
    Utc::now().timestamp()
}

#[cfg(test)]
mod tests {
    use super::{Job, JobPriority, JobType};
    use serde_json::json;

    #[test]
    fn builds_jobs_with_defaults() {
        let job = Job::new(
            JobType::Notification,
            "evt-1",
            "trade-1",
            json!({ "ok": true }),
            JobPriority::High,
        );

        assert_eq!(job.job_type, JobType::Notification);
        assert_eq!(job.priority, JobPriority::High);
        assert_eq!(job.attempts, 0);
        assert_eq!(job.max_attempts, 3);
        assert!(job.run_at.is_none());
        assert!(job.created_at > 0);
    }
}
