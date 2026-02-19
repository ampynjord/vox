use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

use vox_shared::errors::VoxError;
use vox_shared::models::file::{FileRecord, FileInfo, FileStatus, MAX_FILE_SIZE};
use crate::middleware::AuthUser;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upload/:channel_id", post(upload_file))
        .route("/channel/:channel_id", get(list_channel_files))
        .route("/:file_id", get(get_file_info))
        .route("/:file_id", axum::routing::delete(delete_file))
}

/// Upload un fichier dans un canal
async fn upload_file(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<FileInfo>, VoxError> {
    let user_id = claims.sub;

    // Vérifier que le canal existe
    let channel = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM channels WHERE id = $1"
    )
    .bind(channel_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?
    .ok_or(VoxError::NotFound)?;

    let _ = channel;

    // Lire le fichier du multipart
    let field = multipart
        .next_field()
        .await
        .map_err(|_| VoxError::BadRequest("Fichier manquant".into()))?
        .ok_or(VoxError::BadRequest("Aucun fichier envoyé".into()))?;

    let original_filename = field
        .file_name()
        .unwrap_or("unknown")
        .to_string();

    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    let data = field
        .bytes()
        .await
        .map_err(|_| VoxError::BadRequest("Erreur de lecture du fichier".into()))?;

    // Vérifier la taille
    if data.len() as u64 > MAX_FILE_SIZE {
        return Err(VoxError::BadRequest(format!(
            "Fichier trop volumineux (max {} Mo)",
            MAX_FILE_SIZE / 1024 / 1024
        )));
    }

    let file_id = Uuid::new_v4();
    let extension = original_filename
        .rsplit('.')
        .next()
        .unwrap_or("bin");
    let stored_filename = format!("{}.{}", file_id, extension);

    // Créer le répertoire de stockage si nécessaire
    let upload_dir = format!("uploads/{}", channel_id);
    tokio::fs::create_dir_all(&upload_dir)
        .await
        .map_err(|_| VoxError::InternalError)?;

    // Écrire le fichier sur disque
    let file_path = format!("{}/{}", upload_dir, stored_filename);
    tokio::fs::write(&file_path, &data)
        .await
        .map_err(|_| VoxError::InternalError)?;

    let url = format!("/files/{}/{}", channel_id, stored_filename);
    let now = Utc::now();

    // Enregistrer en base
    let record = sqlx::query_as::<_, FileRecord>(
        r#"INSERT INTO files (id, uploader_id, channel_id, filename, original_filename, mime_type, size_bytes, status, url, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
           RETURNING *"#,
    )
    .bind(file_id)
    .bind(user_id)
    .bind(channel_id)
    .bind(&stored_filename)
    .bind(&original_filename)
    .bind(&content_type)
    .bind(data.len() as i64)
    .bind(FileStatus::Ready)
    .bind(&url)
    .bind(now)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Erreur insertion fichier: {}", e);
        VoxError::InternalError
    })?;

    tracing::info!("Fichier {} uploadé par {}", original_filename, user_id);

    Ok(Json(FileInfo::from(record)))
}

#[derive(Deserialize)]
pub struct ListFilesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Lister les fichiers d'un canal
async fn list_channel_files(
    AuthUser(_claims): AuthUser,
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<ListFilesQuery>,
) -> Result<Json<Vec<FileInfo>>, VoxError> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let files = sqlx::query_as::<_, FileRecord>(
        r#"SELECT * FROM files
           WHERE channel_id = $1 AND status = 'ready'
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(channel_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?;

    let infos: Vec<FileInfo> = files.into_iter().map(FileInfo::from).collect();
    Ok(Json(infos))
}

/// Obtenir les infos d'un fichier
async fn get_file_info(
    AuthUser(_claims): AuthUser,
    State(state): State<AppState>,
    Path(file_id): Path<Uuid>,
) -> Result<Json<FileInfo>, VoxError> {
    let file = sqlx::query_as::<_, FileRecord>(
        "SELECT * FROM files WHERE id = $1"
    )
    .bind(file_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?
    .ok_or(VoxError::NotFound)?;

    Ok(Json(FileInfo::from(file)))
}

/// Supprimer un fichier (soft delete)
async fn delete_file(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
    Path(file_id): Path<Uuid>,
) -> Result<StatusCode, VoxError> {
    let file = sqlx::query_as::<_, FileRecord>(
        "SELECT * FROM files WHERE id = $1"
    )
    .bind(file_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?
    .ok_or(VoxError::NotFound)?;

    // Seul l'uploader peut supprimer (ou un admin via permissions, à implémenter)
    if file.uploader_id != claims.sub {
        return Err(VoxError::Forbidden);
    }

    sqlx::query("UPDATE files SET status = 'deleted' WHERE id = $1")
        .bind(file_id)
        .execute(&state.db)
        .await
        .map_err(|_| VoxError::InternalError)?;

    tracing::info!("Fichier {} supprimé par {}", file_id, claims.sub);

    Ok(StatusCode::NO_CONTENT)
}
