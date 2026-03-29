use serde::{Deserialize, Serialize};

/// A transaction envelope ready for signing or broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedTransaction {
    pub xdr: String,
    pub network_passphrase: String,
    pub fee: u32,
    pub sequence: i64,
}

/// A signed transaction ready for submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub xdr: String,
    pub hash: String,
}

/// Queued offline transaction stored locally until connectivity is restored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedTransaction {
    pub id: String,
    pub unsigned_xdr: String,
    pub operation: String,
    pub created_at: u64,
}

/// Minimal trade info optimized for mobile display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileTrade {
    pub id: u64,
    pub seller: String,
    pub buyer: String,
    pub amount: u64,
    pub fee: u64,
    pub status: String,
}

/// Push notification registration payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRegistration {
    pub device_token: String,
    pub platform: Platform,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    Ios,
    Android,
}

/// Mobile-friendly error with retry hint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileError {
    pub code: u32,
    pub message: String,
    pub retryable: bool,
}
