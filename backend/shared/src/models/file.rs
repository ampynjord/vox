use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Fichier uploadé
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FileRecord {
    pub id: Uuid,
    pub uploader_id: Uuid,
    pub message_id: Option<Uuid>,
    pub channel_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub status: FileStatus,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub checksum: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "file_status", rename_all = "lowercase")]
pub enum FileStatus {
    Uploading,
    Ready,
    Failed,
    Deleted,
}

/// Vue publique d'un fichier (pour les API responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<FileRecord> for FileInfo {
    fn from(f: FileRecord) -> Self {
        Self {
            id: f.id,
            filename: f.filename,
            original_filename: f.original_filename,
            mime_type: f.mime_type,
            size_bytes: f.size_bytes,
            url: f.url,
            thumbnail_url: f.thumbnail_url,
            created_at: f.created_at,
        }
    }
}

/// Configuration des limites d'upload
pub const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50 Mo
pub const ALLOWED_IMAGE_TYPES: &[&str] = &["image/jpeg", "image/png", "image/gif", "image/webp"];
pub const ALLOWED_VIDEO_TYPES: &[&str] = &["video/mp4", "video/webm"];
pub const ALLOWED_AUDIO_TYPES: &[&str] = &["audio/mpeg", "audio/ogg", "audio/wav", "audio/webm"];
