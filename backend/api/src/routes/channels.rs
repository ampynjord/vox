use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use uuid::Uuid;

use vox_shared::{
    errors::{VoxError, VoxResult},
    models::channel::*,
    models::space::SpaceMember,
    validation::validate_request,
};

use crate::middleware::AuthUser;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/space/:space_id", get(list_channels).post(create_channel))
        .route("/:channel_id", get(get_channel).put(update_channel).delete(delete_channel))
}

/// POST /api/v1/channels/space/:space_id
async fn create_channel(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(space_id): Path<Uuid>,
    Json(req): Json<CreateChannelRequest>,
) -> VoxResult<Json<Channel>> {
    validate_request(&req)?;

    // Vérifier que l'utilisateur est membre avec un rôle suffisant
    let _member = sqlx::query_as::<_, SpaceMember>(
        "SELECT * FROM space_members WHERE space_id = $1 AND user_id = $2 AND role IN ('owner', 'admin', 'moderator')"
    )
    .bind(space_id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::Forbidden)?;

    // Trouver la position maximale actuelle
    let max_pos: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(position) FROM channels WHERE space_id = $1"
    )
    .bind(space_id)
    .fetch_one(&state.db)
    .await?;

    let channel = sqlx::query_as::<_, Channel>(
        r#"
        INSERT INTO channels (id, space_id, name, description, channel_type, position, is_ephemeral, ephemeral_duration_hours)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(space_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(req.channel_type.unwrap_or_default())
    .bind(max_pos.unwrap_or(0) + 1)
    .bind(req.is_ephemeral.unwrap_or(false))
    .bind(req.ephemeral_duration_hours)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(channel))
}

/// GET /api/v1/channels/space/:space_id
async fn list_channels(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(space_id): Path<Uuid>,
) -> VoxResult<Json<Vec<Channel>>> {
    // Vérifier l'appartenance à l'espace
    let _member = sqlx::query_as::<_, SpaceMember>(
        "SELECT * FROM space_members WHERE space_id = $1 AND user_id = $2"
    )
    .bind(space_id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::Forbidden)?;

    let channels = sqlx::query_as::<_, Channel>(
        "SELECT * FROM channels WHERE space_id = $1 ORDER BY position"
    )
    .bind(space_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(channels))
}

/// GET /api/v1/channels/:channel_id
async fn get_channel(
    State(state): State<AppState>,
    AuthUser(_claims): AuthUser,
    Path(channel_id): Path<Uuid>,
) -> VoxResult<Json<Channel>> {
    let channel = sqlx::query_as::<_, Channel>(
        "SELECT * FROM channels WHERE id = $1"
    )
    .bind(channel_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::NotFound)?;

    Ok(Json(channel))
}

/// PUT /api/v1/channels/:channel_id
async fn update_channel(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(channel_id): Path<Uuid>,
    Json(req): Json<UpdateChannelRequest>,
) -> VoxResult<Json<Channel>> {
    validate_request(&req)?;

    // Récupérer le canal et vérifier les permissions
    let channel = sqlx::query_as::<_, Channel>(
        "SELECT * FROM channels WHERE id = $1"
    )
    .bind(channel_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::NotFound)?;

    let _member = sqlx::query_as::<_, SpaceMember>(
        "SELECT * FROM space_members WHERE space_id = $1 AND user_id = $2 AND role IN ('owner', 'admin', 'moderator')"
    )
    .bind(channel.space_id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::Forbidden)?;

    let updated = sqlx::query_as::<_, Channel>(
        r#"
        UPDATE channels SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            position = COALESCE($4, position)
        WHERE id = $1
        RETURNING *
        "#
    )
    .bind(channel_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(req.position)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated))
}

/// DELETE /api/v1/channels/:channel_id
async fn delete_channel(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(channel_id): Path<Uuid>,
) -> VoxResult<Json<serde_json::Value>> {
    let channel = sqlx::query_as::<_, Channel>(
        "SELECT * FROM channels WHERE id = $1"
    )
    .bind(channel_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::NotFound)?;

    let _member = sqlx::query_as::<_, SpaceMember>(
        "SELECT * FROM space_members WHERE space_id = $1 AND user_id = $2 AND role IN ('owner', 'admin')"
    )
    .bind(channel.space_id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::Forbidden)?;

    sqlx::query("DELETE FROM channels WHERE id = $1")
        .bind(channel_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"message": "Canal supprimé"})))
}
