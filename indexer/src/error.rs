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
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Database error"),
            AppError::StellarSdk(_) => (StatusCode::INTERNAL_SERVER_ERROR, "STELLAR_ERROR", "Stellar network error"),
            AppError::Serialization(_) => (StatusCode::BAD_REQUEST, "INVALID_FORMAT", "Invalid data format"),
            AppError::HttpClient(_) => (StatusCode::INTERNAL_SERVER_ERROR, "NETWORK_ERROR", "Network error"),
            AppError::InvalidEventData(_) => (StatusCode::BAD_REQUEST, "INVALID_EVENT_DATA", "Invalid event data"),
            AppError::EventNotFound => (StatusCode::NOT_FOUND, "EVENT_NOT_FOUND", "Event not found"),
            AppError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error"),
        };

        let detail = self.to_string();
        let body = Json(json!({
            "error": { "code": code, "message": message, "detail": detail }
        }));

        (status, body).into_response()
    }
}