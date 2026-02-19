use axum::{
    extract::{Path, State},
    Json, Router,
    routing::get,
};
use serde::Serialize;
use uuid::Uuid;
use redis::AsyncCommands;

use vox_shared::errors::VoxError;
use crate::middleware::AuthUser;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/channels/:channel_id/participants", get(get_voice_participants))
        .route("/channels/:channel_id/info", get(get_voice_channel_info))
}

#[derive(Serialize)]
struct VoiceParticipant {
    user_id: Uuid,
    username: String,
    is_muted: bool,
    is_deafened: bool,
    is_screen_sharing: bool,
}

#[derive(Serialize)]
struct VoiceChannelInfo {
    channel_id: Uuid,
    participants: Vec<VoiceParticipant>,
    participant_count: usize,
}

/// Lister les participants d'un canal vocal (via Redis)
async fn get_voice_participants(
    AuthUser(_claims): AuthUser,
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<Vec<VoiceParticipant>>, VoxError> {
    let mut redis = state.redis.clone();

    // Scanner les clés Redis pour trouver les participants
    let pattern = format!("voice:{}:*", channel_id);

    // Utiliser KEYS (pour le dev; en production on utilisera SCAN)
    let keys: Vec<String> = redis.keys(&pattern)
        .await
        .map_err(|_| VoxError::InternalError)?;

    let mut participants = Vec::new();
    for key in keys {
        // Extraire le user_id de la clé "voice:{channel_id}:{user_id}"
        if let Some(uid_str) = key.rsplit(':').next() {
            if let Ok(user_id) = Uuid::parse_str(uid_str) {
                // Récupérer le username depuis la DB
                let username: Option<String> = sqlx::query_scalar(
                    "SELECT username FROM users WHERE id = $1"
                )
                .bind(user_id)
                .fetch_optional(&state.db)
                .await
                .map_err(|_| VoxError::InternalError)?;

                if let Some(username) = username {
                    participants.push(VoiceParticipant {
                        user_id,
                        username,
                        is_muted: false,
                        is_deafened: false,
                        is_screen_sharing: false,
                    });
                }
            }
        }
    }

    Ok(Json(participants))
}

/// Obtenir les informations d'un canal vocal
async fn get_voice_channel_info(
    AuthUser(_claims): AuthUser,
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<VoiceChannelInfo>, VoxError> {
    // Vérifier que le canal existe et est de type vocal
    let channel_exists = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM channels WHERE id = $1"
    )
    .bind(channel_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?
    .is_some();

    if !channel_exists {
        return Err(VoxError::NotFound);
    }

    let mut redis = state.redis.clone();
    let pattern = format!("voice:{}:*", channel_id);
    let keys: Vec<String> = redis.keys(&pattern)
        .await
        .map_err(|_| VoxError::InternalError)?;

    let mut participants = Vec::new();
    for key in keys {
        if let Some(uid_str) = key.rsplit(':').next() {
            if let Ok(user_id) = Uuid::parse_str(uid_str) {
                let username: Option<String> = sqlx::query_scalar(
                    "SELECT username FROM users WHERE id = $1"
                )
                .bind(user_id)
                .fetch_optional(&state.db)
                .await
                .map_err(|_| VoxError::InternalError)?;

                if let Some(username) = username {
                    participants.push(VoiceParticipant {
                        user_id,
                        username,
                        is_muted: false,
                        is_deafened: false,
                        is_screen_sharing: false,
                    });
                }
            }
        }
    }

    let count = participants.len();
    Ok(Json(VoiceChannelInfo {
        channel_id,
        participants,
        participant_count: count,
    }))
}
