use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use vox_shared::{
    errors::{VoxError, VoxResult},
    models::message::*,
    validation::validate_request,
};

use crate::middleware::AuthUser;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    pub before: Option<Uuid>,
    pub limit: Option<i64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/channel/:channel_id", get(list_messages).post(create_message))
        .route("/:message_id", put(update_message).delete(delete_message))
}

/// POST /api/v1/messages/channel/:channel_id
async fn create_message(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(channel_id): Path<Uuid>,
    Json(req): Json<CreateMessageRequest>,
) -> VoxResult<Json<MessageWithAuthor>> {
    validate_request(&req)?;

    // Vérifier que le canal existe
    let _channel = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM channels WHERE id = $1"
    )
    .bind(channel_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::NotFound)?;

    // Créer le message et récupérer avec les infos auteur
    let msg = sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (id, channel_id, author_id, content, reply_to)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(channel_id)
    .bind(claims.sub)
    .bind(&req.content)
    .bind(req.reply_to)
    .fetch_one(&state.db)
    .await?;

    // Récupérer le message avec les infos auteur
    let message = sqlx::query_as::<_, MessageWithAuthor>(
        r#"
        SELECT m.id, m.channel_id, m.author_id, u.username as author_username,
               u.display_name as author_display_name, u.avatar_url as author_avatar_url,
               m.content, m.edited, m.reply_to, m.created_at, m.updated_at
        FROM messages m
        INNER JOIN users u ON m.author_id = u.id
        WHERE m.id = $1
        "#
    )
    .bind(msg.id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(message))
}

/// GET /api/v1/messages/channel/:channel_id
async fn list_messages(
    State(state): State<AppState>,
    AuthUser(_claims): AuthUser,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<MessageQuery>,
) -> VoxResult<Json<Vec<MessageWithAuthor>>> {
    let limit = query.limit.unwrap_or(50).min(100);

    let messages = if let Some(before) = query.before {
        sqlx::query_as::<_, MessageWithAuthor>(
            r#"
            SELECT m.id, m.channel_id, m.author_id, u.username as author_username,
                   u.display_name as author_display_name, u.avatar_url as author_avatar_url,
                   m.content, m.edited, m.reply_to, m.created_at, m.updated_at
            FROM messages m
            INNER JOIN users u ON m.author_id = u.id
            WHERE m.channel_id = $1 AND m.created_at < (SELECT created_at FROM messages WHERE id = $2)
            ORDER BY m.created_at DESC
            LIMIT $3
            "#
        )
        .bind(channel_id)
        .bind(before)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, MessageWithAuthor>(
            r#"
            SELECT m.id, m.channel_id, m.author_id, u.username as author_username,
                   u.display_name as author_display_name, u.avatar_url as author_avatar_url,
                   m.content, m.edited, m.reply_to, m.created_at, m.updated_at
            FROM messages m
            INNER JOIN users u ON m.author_id = u.id
            WHERE m.channel_id = $1
            ORDER BY m.created_at DESC
            LIMIT $2
            "#
        )
        .bind(channel_id)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(messages))
}

/// PUT /api/v1/messages/:message_id
async fn update_message(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(message_id): Path<Uuid>,
    Json(req): Json<UpdateMessageRequest>,
) -> VoxResult<Json<MessageWithAuthor>> {
    validate_request(&req)?;

    // Vérifier que le message appartient à l'utilisateur
    let msg = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE id = $1"
    )
    .bind(message_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::NotFound)?;

    if msg.author_id != claims.sub {
        return Err(VoxError::Forbidden);
    }

    sqlx::query(
        "UPDATE messages SET content = $2, edited = true WHERE id = $1"
    )
    .bind(message_id)
    .bind(&req.content)
    .execute(&state.db)
    .await?;

    let message = sqlx::query_as::<_, MessageWithAuthor>(
        r#"
        SELECT m.id, m.channel_id, m.author_id, u.username as author_username,
               u.display_name as author_display_name, u.avatar_url as author_avatar_url,
               m.content, m.edited, m.reply_to, m.created_at, m.updated_at
        FROM messages m
        INNER JOIN users u ON m.author_id = u.id
        WHERE m.id = $1
        "#
    )
    .bind(message_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(message))
}

/// DELETE /api/v1/messages/:message_id
async fn delete_message(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(message_id): Path<Uuid>,
) -> VoxResult<Json<serde_json::Value>> {
    let msg = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE id = $1"
    )
    .bind(message_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::NotFound)?;

    if msg.author_id != claims.sub {
        return Err(VoxError::Forbidden);
    }

    sqlx::query("DELETE FROM messages WHERE id = $1")
        .bind(message_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"message": "Message supprimé"})))
}
