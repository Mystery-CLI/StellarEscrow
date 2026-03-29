use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
};
use bytes::Bytes;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::FileListQuery;
use crate::storage::{FileCategory, StorageService};

/// POST /files/:category
/// Multipart fields: `file` (binary), `owner_id` (text), `trade_id` (optional text)
pub async fn upload_file(
    Path(category_str): Path<String>,
    State(storage): State<Arc<StorageService>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let category = FileCategory::from_str(&category_str).ok_or(AppError::InvalidFileCategory)?;

    let mut file_data: Option<Bytes> = None;
    let mut original_name = String::from("upload");
    let mut mime_type = String::from("application/octet-stream");
    let mut owner_id = String::new();
    let mut trade_id: Option<i64> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Storage(format!("Multipart error: {e}")))?
    {
        match field.name() {
            Some("file") => {
                if let Some(fname) = field.file_name() {
                    original_name = fname.to_string();
                }
                if let Some(ct) = field.content_type() {
                    mime_type = ct.to_string();
                }
                file_data =
                    Some(field.bytes().await.map_err(|e| {
                        AppError::Storage(format!("Failed to read field bytes: {e}"))
                    })?);
            }
            Some("owner_id") => {
                owner_id = field
                    .text()
                    .await
                    .map_err(|e| AppError::Storage(format!("Failed to read owner_id: {e}")))?;
            }
            Some("trade_id") => {
                let raw = field.text().await.unwrap_or_default();
                trade_id = raw.parse::<i64>().ok();
            }
            _ => {}
        }
    }

    if owner_id.is_empty() {
        return Err(AppError::Storage("owner_id is required".into()));
    }
    let data = file_data.ok_or_else(|| AppError::Storage("No file field in request".into()))?;

    let record = storage
        .upload(
            &owner_id,
            category,
            &original_name,
            &mime_type,
            data,
            trade_id,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(json!({ "file": record }))))
}

/// GET /files/:id?requester_id=<addr>
pub async fn download_file(
    Path(file_id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(storage): State<Arc<StorageService>>,
) -> Result<Response, AppError> {
    let requester_id = params.get("requester_id").cloned().unwrap_or_default();

    let (record, data) = storage.download(file_id, &requester_id).await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &record.mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", record.original_name),
        )
        .header(header::CONTENT_LENGTH, data.len())
        .body(Body::from(data))
        .map_err(|e| AppError::Storage(format!("Response build error: {e}")))?;

    Ok(response)
}

/// DELETE /files/:id?requester_id=<addr>
pub async fn delete_file(
    Path(file_id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(storage): State<Arc<StorageService>>,
) -> Result<impl IntoResponse, AppError> {
    let requester_id = params.get("requester_id").cloned().unwrap_or_default();

    storage.delete(file_id, &requester_id).await?;
    Ok(Json(json!({ "deleted": file_id })))
}

/// GET /files?owner_id=<addr>&file_type=<category>
pub async fn list_files(
    Query(params): Query<FileListQuery>,
    State(storage): State<Arc<StorageService>>,
) -> Result<impl IntoResponse, AppError> {
    let records = storage
        .list_by_owner(&params.owner_id, params.file_type.as_deref())
        .await?;
    Ok(Json(json!({ "files": records })))
}
