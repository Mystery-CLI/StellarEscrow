use std::collections::VecDeque;

use crate::types::QueuedTransaction;

/// In-memory offline queue. On mobile, persist this to local storage between sessions.
pub struct OfflineQueue {
    queue: VecDeque<QueuedTransaction>,
}

impl OfflineQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Enqueue a transaction for later submission when online.
    pub fn enqueue(&mut self, tx: QueuedTransaction) {
        self.queue.push_back(tx);
    }

    /// Drain all queued transactions for submission.
    pub fn drain(&mut self) -> Vec<QueuedTransaction> {
        self.queue.drain(..).collect()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Serialize queue to JSON for persistent local storage.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let items: Vec<&QueuedTransaction> = self.queue.iter().collect();
        serde_json::to_string(&items)
    }

    /// Restore queue from JSON (call on app startup).
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let items: Vec<QueuedTransaction> = serde_json::from_str(json)?;
        Ok(Self {
            queue: VecDeque::from(items),
        })
    }
}

impl Default for OfflineQueue {
    fn default() -> Self {
        Self::new()
    }
}
