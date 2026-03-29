use bytes::Bytes;
use chrono::Utc;
use image::ImageFormat;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::io::Cursor;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::FileRecord;

/// Maximum file sizes per category (bytes)
const MAX_AVATAR_SIZE: usize = 2 * 1024 * 1024; // 2 MB
const MAX_EVIDENCE_SIZE: usize = 20 * 1024 * 1024; // 20 MB
const MAX_DOCUMENT_SIZE: usize = 10 * 1024 * 1024; // 10 MB

/// Allowed MIME types per category
const AVATAR_MIMES: &[&str] = &["image/jpeg", "image/png", "image/webp"];
const EVIDENCE_MIMES: &[&str] = &["image/jpeg", "image/png", "image/webp", "application/pdf"];
const DOCUMENT_MIMES: &[&str] = &[
    "application/pdf",
    "text/plain",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
];

#[derive(Debug, Clone, PartialEq)]
pub enum FileCategory {
    Avatar,
    Evidence,
    Document,
}

impl FileCategory {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "avatar" => Some(Self::Avatar),
            "evidence" => Some(Self::Evidence),
            "document" => Some(Self::Document),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Avatar => "avatar",
            Self::Evidence => "evidence",
            Self::Document => "document",
        }
    }

    fn max_size(&self) -> usize {
        match self {
            Self::Avatar => MAX_AVATAR_SIZE,
            Self::Evidence => MAX_EVIDENCE_SIZE,
            Self::Document => MAX_DOCUMENT_SIZE,
        }
    }

    fn allowed_mimes(&self) -> &'static [&'static str] {
        match self {
            Self::Avatar => AVATAR_MIMES,
            Self::Evidence => EVIDENCE_MIMES,
            Self::Document => DOCUMENT_MIMES,
        }
    }
}

pub struct StorageService {
    pool: PgPool,
    base_dir: PathBuf,
}

impl StorageService {
    pub async fn new(pool: PgPool, base_dir: impl Into<PathBuf>) -> Result<Self, AppError> {
        let base_dir = base_dir.into();
        for sub in &["avatar", "evidence", "document"] {
            fs::create_dir_all(base_dir.join(sub))
                .await
                .map_err(|e| AppError::Storage(format!("Failed to create storage dir: {e}")))?;
        }
        Ok(Self { pool, base_dir })
    }

    /// Validate, optionally compress, store, and record a file.
    pub async fn upload(
        &self,
        owner_id: &str,
        category: FileCategory,
        original_name: &str,
        mime_type: &str,
        data: Bytes,
        trade_id: Option<i64>,
    ) -> Result<FileRecord, AppError> {
        // --- Validation ---
        self.validate(&category, mime_type, data.len())?;

        // --- Optional compression for images ---
        let (final_data, compressed) = self.maybe_compress(&category, mime_type, data)?;

        // --- Checksum ---
        let checksum = hex::encode(Sha256::digest(&final_data));

        // --- Persist to disk ---
        let id = Uuid::new_v4();
        let ext = extension_for(mime_type);
        let stored_name = format!("{}.{}", id, ext);
        let dest = self.base_dir.join(category.as_str()).join(&stored_name);
        fs::write(&dest, &final_data)
            .await
            .map_err(|e| AppError::Storage(format!("Failed to write file: {e}")))?;

        // --- DB record ---
        let record = FileRecord {
            id,
            owner_id: owner_id.to_string(),
            file_type: category.as_str().to_string(),
            original_name: original_name.to_string(),
            stored_name: stored_name.clone(),
            mime_type: mime_type.to_string(),
            size_bytes: final_data.len() as i64,
            checksum,
            trade_id,
            is_compressed: compressed,
            created_at: Utc::now(),
        };

        sqlx::query(
            r#"INSERT INTO files
               (id, owner_id, file_type, original_name, stored_name, mime_type,
                size_bytes, checksum, trade_id, is_compressed, created_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"#,
        )
        .bind(record.id)
        .bind(&record.owner_id)
        .bind(&record.file_type)
        .bind(&record.original_name)
        .bind(&record.stored_name)
        .bind(&record.mime_type)
        .bind(record.size_bytes)
        .bind(&record.checksum)
        .bind(record.trade_id)
        .bind(record.is_compressed)
        .bind(record.created_at)
        .execute(&self.pool)
        .await?;

        Ok(record)
    }

    /// Download: returns raw bytes after access-control check.
    pub async fn download(
        &self,
        file_id: Uuid,
        requester_id: &str,
    ) -> Result<(FileRecord, Bytes), AppError> {
        let record = self.get_record(file_id).await?;
        self.check_access(&record, requester_id)?;

        let path = self
            .base_dir
            .join(&record.file_type)
            .join(&record.stored_name);
        let data = fs::read(&path)
            .await
            .map_err(|e| AppError::Storage(format!("Failed to read file: {e}")))?;

        Ok((record, Bytes::from(data)))
    }

    /// Delete a file (owner only).
    pub async fn delete(&self, file_id: Uuid, requester_id: &str) -> Result<(), AppError> {
        let record = self.get_record(file_id).await?;
        if record.owner_id != requester_id {
            return Err(AppError::Forbidden(
                "Only the owner can delete this file".into(),
            ));
        }

        let path = self
            .base_dir
            .join(&record.file_type)
            .join(&record.stored_name);
        let _ = fs::remove_file(&path).await; // best-effort

        sqlx::query("DELETE FROM files WHERE id = $1")
            .bind(file_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// List files owned by a given address, optionally filtered by category.
    pub async fn list_by_owner(
        &self,
        owner_id: &str,
        category: Option<&str>,
    ) -> Result<Vec<FileRecord>, AppError> {
        let records = if let Some(cat) = category {
            sqlx::query_as::<_, FileRecord>(
                "SELECT * FROM files WHERE owner_id = $1 AND file_type = $2 ORDER BY created_at DESC",
            )
            .bind(owner_id)
            .bind(cat)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, FileRecord>(
                "SELECT * FROM files WHERE owner_id = $1 ORDER BY created_at DESC",
            )
            .bind(owner_id)
            .fetch_all(&self.pool)
            .await?
        };
        Ok(records)
    }

    // ---- private helpers ----

    async fn get_record(&self, file_id: Uuid) -> Result<FileRecord, AppError> {
        sqlx::query_as::<_, FileRecord>("SELECT * FROM files WHERE id = $1")
            .bind(file_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::FileNotFound)
    }

    fn validate(
        &self,
        category: &FileCategory,
        mime_type: &str,
        size: usize,
    ) -> Result<(), AppError> {
        if size > category.max_size() {
            return Err(AppError::FileTooLarge(category.max_size()));
        }
        if !category.allowed_mimes().contains(&mime_type) {
            return Err(AppError::InvalidMimeType(mime_type.to_string()));
        }
        Ok(())
    }

    fn maybe_compress(
        &self,
        category: &FileCategory,
        mime_type: &str,
        data: Bytes,
    ) -> Result<(Bytes, bool), AppError> {
        // Only compress images for avatar and evidence
        let is_image = matches!(mime_type, "image/jpeg" | "image/png" | "image/webp");
        if !is_image || *category == FileCategory::Document {
            return Ok((data, false));
        }

        let img = image::load_from_memory(&data)
            .map_err(|e| AppError::Storage(format!("Image decode error: {e}")))?;

        // Resize avatar to max 256x256
        let img = if *category == FileCategory::Avatar {
            img.thumbnail(256, 256)
        } else {
            // Evidence: cap longest side at 1920
            if img.width() > 1920 {
                img.thumbnail(1920, 1920)
            } else {
                img
            }
        };

        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Jpeg)
            .map_err(|e| AppError::Storage(format!("Image encode error: {e}")))?;

        Ok((Bytes::from(buf.into_inner()), true))
    }

    /// Access control: avatars are public; evidence/documents require ownership or trade participation.
    fn check_access(&self, record: &FileRecord, requester_id: &str) -> Result<(), AppError> {
        if record.file_type == "avatar" {
            return Ok(()); // avatars are public
        }
        if record.owner_id == requester_id {
            return Ok(());
        }
        Err(AppError::Forbidden("Access denied".into()))
    }
}

fn extension_for(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "application/pdf" => "pdf",
        "text/plain" => "txt",
        _ => "bin",
    }
}
