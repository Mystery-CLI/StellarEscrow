use thiserror::Error;

#[derive(Debug, Error)]
pub enum MobileSdkError {
    #[error("network unavailable — transaction queued for offline signing")]
    Offline,
    #[error("invalid keypair: {0}")]
    InvalidKeypair(String),
    #[error("transaction build failed: {0}")]
    BuildFailed(String),
    #[error("submission failed (code {code}): {message}")]
    SubmissionFailed { code: u32, message: String },
    #[error("invalid contract response: {0}")]
    InvalidResponse(String),
    #[error("push notification registration failed: {0}")]
    PushRegistrationFailed(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}
