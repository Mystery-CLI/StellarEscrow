use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Stellar SDK error: {0}")]
    StellarSdk(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Invalid event data: {0}")]
    InvalidEventData(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Event not found")]
    EventNotFound,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error")]
    InternalServerError,

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("File not found")]
    FileNotFound,

    #[error("File too large (max {0} bytes)")]
    FileTooLarge(usize),

    #[error("Invalid MIME type: {0}")]
    InvalidMimeType(String),

    #[error("Invalid file category")]
    InvalidFileCategory,

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "Database error",
            ),
            AppError::StellarSdk(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "STELLAR_ERROR",
                "Stellar network error",
            ),
            AppError::Serialization(_) => (
                StatusCode::BAD_REQUEST,
                "INVALID_FORMAT",
                "Invalid data format",
            ),
            AppError::HttpClient(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "NETWORK_ERROR",
                "Network error",
            ),
            AppError::InvalidEventData(_) => (
                StatusCode::BAD_REQUEST,
                "INVALID_EVENT_DATA",
                "Invalid event data",
            ),
            AppError::BadRequest(_) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                "Bad request",
            ),
            AppError::EventNotFound => {
                (StatusCode::NOT_FOUND, "EVENT_NOT_FOUND", "Event not found")
            }
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND", "Resource not found"),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            ),
            AppError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                "Rate limit exceeded",
            ),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN", "Forbidden"),
            AppError::FileNotFound => (StatusCode::NOT_FOUND, "FILE_NOT_FOUND", "File not found"),
            AppError::FileTooLarge(_) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "FILE_TOO_LARGE",
                "File too large",
            ),
            AppError::InvalidMimeType(_) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "INVALID_MIME",
                "Unsupported file type",
            ),
            AppError::InvalidFileCategory => (
                StatusCode::BAD_REQUEST,
                "INVALID_CATEGORY",
                "Invalid file category",
            ),
            AppError::Storage(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "STORAGE_ERROR",
                "Storage error",
            ),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT", "Resource already exists"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", "Bad request"),
        };

        let detail = self.to_string();
        let body = Json(json!({
            "error": { "code": code, "message": message, "detail": detail }
        }));

        (status, body).into_response()
    }
}
