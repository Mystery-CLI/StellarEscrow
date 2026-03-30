use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use bytes::Bytes;
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{DisputeEvidenceRecord, DisputeEvidenceResponse, EvidenceDownloadToken};
use crate::storage::{FileCategory, StorageService};
use crate::database::Database;

#[derive(Clone)]
pub struct EvidenceState {
    pub storage: Arc<StorageService>,
    pub db: Arc<Database>,
}

#[derive(Deserialize)]
pub struct EvidenceAccessQuery {
    pub requester: String,
}

/// POST /disputes/:id/evidence
/// Multipart: `file` (binary), `uploader` (Stellar address), `description` (optional text)
pub async fn upload_dispute_evidence(
    Path(dispute_id): Path<i64>,
    State(state): State<EvidenceState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut file_data: Option<Bytes> = None;
    let mut original_name = String::from("evidence");
    let mut mime_type = String::from("application/octet-stream");
    let mut uploader = String::new();
    let mut description: Option<String> = None;

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
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::Storage(format!("Failed to read file: {e}")))?,
                );
            }
            Some("uploader") => {
                uploader = field
                    .text()
                    .await
                    .map_err(|e| AppError::Storage(format!("Failed to read uploader: {e}")))?;
            }
            Some("description") => {
                let text = field.text().await.unwrap_or_default();
                if !text.is_empty() {
                    description = Some(text);
                }
            }
            _ => {}
        }
    }

    if uploader.is_empty() {
        return Err(AppError::BadRequest("uploader address is required".into()));
    }
    let data = file_data.ok_or_else(|| AppError::BadRequest("No file field in request".into()))?;

    // Verify uploader is a participant or arbitrator for this dispute
    state
        .db
        .check_dispute_participant(dispute_id, &uploader)
        .await?;

    // Store the file under the "evidence" category
    let file_record = state
        .storage
        .upload(&uploader, FileCategory::Evidence, &original_name, &mime_type, data, Some(dispute_id))
        .await?;

    // Record the dispute evidence link
    let evidence = state
        .db
        .insert_dispute_evidence(dispute_id, file_record.id, &uploader, description.as_deref())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "evidence": evidence,
            "file": file_record,
        })),
    ))
}

/// GET /disputes/:id/evidence
/// Returns all evidence for a dispute (participants/arbitrators only).
pub async fn list_dispute_evidence(
    Path(dispute_id): Path<i64>,
    Query(params): Query<EvidenceAccessQuery>,
    State(state): State<EvidenceState>,
) -> Result<impl IntoResponse, AppError> {
    state
        .db
        .check_dispute_participant(dispute_id, &params.requester)
        .await?;

    let items = state.db.list_dispute_evidence(dispute_id).await?;
    Ok(Json(json!({ "evidence": items })))
}

/// GET /disputes/:id/evidence/:evidence_id/download-url?requester=<addr>
/// Issues a short-lived signed token for downloading a specific evidence file.
pub async fn get_evidence_download_url(
    Path((dispute_id, evidence_id)): Path<(i64, Uuid)>,
    Query(params): Query<EvidenceAccessQuery>,
    State(state): State<EvidenceState>,
) -> Result<impl IntoResponse, AppError> {
    state
        .db
        .check_dispute_participant(dispute_id, &params.requester)
        .await?;

    // Verify the evidence belongs to this dispute
    let evidence = state
        .db
        .get_dispute_evidence(evidence_id)
        .await?
        .ok_or(AppError::NotFound("Evidence not found".into()))?;

    if evidence.dispute_id != dispute_id {
        return Err(AppError::Forbidden("Evidence does not belong to this dispute".into()));
    }

    let token = state
        .db
        .create_evidence_download_token(evidence.file_id, &params.requester)
        .await?;

    Ok(Json(json!({
        "token": token.token,
        "expires_at": token.expires_at,
        "download_url": format!("/evidence/download/{}", token.token),
    })))
}

/// GET /evidence/download/:token
/// Redeems a signed download token and streams the file.
pub async fn redeem_evidence_download(
    Path(token): Path<Uuid>,
    State(state): State<EvidenceState>,
) -> Result<impl IntoResponse, AppError> {
    let token_record = state
        .db
        .consume_evidence_download_token(token)
        .await?
        .ok_or(AppError::NotFound("Invalid or expired download token".into()))?;

    let (record, data) = state
        .storage
        .download(token_record.file_id, &token_record.requester)
        .await?;

    let response = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, &record.mime_type)
        .header(
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", record.original_name),
        )
        .header(axum::http::header::CONTENT_LENGTH, data.len())
        .body(axum::body::Body::from(data))
        .map_err(|e| AppError::Storage(format!("Response build error: {e}")))?;

    Ok(response)
}
