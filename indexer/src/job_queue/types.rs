use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    Event,
    Notification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub job_type: JobType,
    pub event_id: String,
    pub trade_id: String,
    pub payload: Value,
    #[serde(default)]
    pub priority: JobPriority,
    /// Unix timestamp (seconds). None means run immediately.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum JobPriority {
    High,
    #[default]
    Normal,
}
