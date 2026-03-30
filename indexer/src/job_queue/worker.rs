use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::error::AppError;

use super::types::{Job, JobMonitorSnapshot, JobPriority, JobType};
use super::JobQueue;

#[derive(Clone)]
pub struct JobWorker {
    queue: Arc<tokio::sync::Mutex<JobQueue>>,
    state: Arc<RwLock<JobWorkerState>>,
}

#[derive(Debug, Clone)]
struct JobWorkerState {
    name: String,
    running: bool,
    processed_jobs: u64,
    failed_jobs: u64,
}

impl JobWorker {
    pub fn new(queue: Arc<tokio::sync::Mutex<JobQueue>>, name: impl Into<String>) -> Self {
        Self {
            queue,
            state: Arc::new(RwLock::new(JobWorkerState {
                name: name.into(),
                running: false,
                processed_jobs: 0,
                failed_jobs: 0,
            })),
        }
    }

    pub async fn run(self) -> Result<(), AppError> {
        {
            let mut state = self.state.write().await;
            state.running = true;
        }

        loop {
            let now = chrono::Utc::now().timestamp();
            {
                let mut queue = self.queue.lock().await;
                let promoted = queue.promote_scheduled(now).await?;
                if promoted > 0 {
                    info!("[job-worker] promoted {} scheduled jobs", promoted);
                }
            }

            let job = {
                let mut queue = self.queue.lock().await;
                queue.dequeue().await?
            };

            let Some(job) = job else {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            };

            if let Err(err) = self.process_with_retry(job).await {
                error!("[job-worker] unrecoverable worker error: {}", err);
            }
        }
    }

    pub async fn snapshot(&self) -> Result<JobMonitorSnapshot, AppError> {
        let state = self.state.read().await.clone();
        let mut queue = self.queue.lock().await;
        queue.snapshot(&state.name, state.running).await
    }

    async fn process_with_retry(&self, mut job: Job) -> Result<(), AppError> {
        loop {
            match self.process(&job).await {
                Ok(()) => {
                    {
                        let mut state = self.state.write().await;
                        state.processed_jobs += 1;
                    }
                    let mut queue = self.queue.lock().await;
                    queue.record_processed(&job).await?;
                    info!(
                        "[job-worker] job {} completed with priority {}",
                        job.id,
                        job.priority.as_str()
                    );
                    return Ok(());
                }
                Err(err) => {
                    job.attempts += 1;
                    job.last_error = Some(err.to_string());

                    if job.attempts >= job.max_attempts {
                        {
                            let mut state = self.state.write().await;
                            state.failed_jobs += 1;
                        }
                        let mut queue = self.queue.lock().await;
                        queue.dead_letter(&job).await?;
                        error!(
                            "[job-worker] job {} moved to dead letter after {} attempts: {}",
                            job.id, job.attempts, err
                        );
                        return Ok(());
                    }

                    let delay_secs = 2_i64.pow(job.attempts.saturating_sub(1)) as i64;
                    let next_run = chrono::Utc::now().timestamp() + delay_secs;
                    job.run_at = Some(next_run);
                    job.priority = retry_priority(&job.priority);

                    let mut queue = self.queue.lock().await;
                    queue.record_retry(&job).await?;
                    queue.enqueue_at(job.clone(), next_run).await?;
                    warn!(
                        "[job-worker] job {} failed on attempt {}/{}: {}. rescheduled for {}",
                        job.id, job.attempts, job.max_attempts, err, next_run
                    );
                    return Ok(());
                }
            }
        }
    }

    async fn process(&self, job: &Job) -> Result<(), AppError> {
        match job.job_type {
            JobType::Event => {
                info!("[job-worker] processing event job {}", job.id);
                Ok(())
            }
            JobType::Notification => {
                info!("[job-worker] processing notification job {}", job.id);
                Ok(())
            }
            JobType::CacheWarm => {
                info!("[job-worker] processing cache warm job {}", job.id);
                Ok(())
            }
            JobType::Compliance => {
                info!("[job-worker] processing compliance job {}", job.id);
                Ok(())
            }
        }
    }
}

fn retry_priority(priority: &JobPriority) -> JobPriority {
    match priority {
        JobPriority::Critical => JobPriority::High,
        JobPriority::High => JobPriority::High,
        JobPriority::Normal => JobPriority::Normal,
        JobPriority::Low => JobPriority::Low,
    }
}
