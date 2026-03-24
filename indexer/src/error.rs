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

    #[error("Event not found")]
    EventNotFound,

    #[error("Internal server error")]
    InternalServerError,

    #[error("File not found")]
    FileNotFound,

    #[error("File too large (max {0} bytes)")]
    FileTooLarge(usize),

    #[error("Invalid MIME type: {0}")]
    InvalidMimeType(String),

    #[error("Invalid file category")]
    InvalidFileCategory,

    #[error("Access denied: {0}")]
    Forbidden(String),

    #[error("Storage error: {0}")]
    Storage(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::StellarSdk(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Stellar network error"),
            AppError::Serialization(_) => (StatusCode::BAD_REQUEST, "Invalid data format"),
            AppError::HttpClient(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Network error"),
            AppError::InvalidEventData(_) => (StatusCode::BAD_REQUEST, "Invalid event data"),
            AppError::EventNotFound => (StatusCode::NOT_FOUND, "Event not found"),
            AppError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            AppError::FileNotFound => (StatusCode::NOT_FOUND, "File not found"),
            AppError::FileTooLarge(_) => (StatusCode::PAYLOAD_TOO_LARGE, "File too large"),
            AppError::InvalidMimeType(_) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported file type"),
            AppError::InvalidFileCategory => (StatusCode::BAD_REQUEST, "Invalid file category"),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "Access denied"),
            AppError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Storage error"),
        };

        let body = Json(json!({
            "error": error_message,
            "message": self.to_string()
        }));

        (status, body).into_response()
    }
}