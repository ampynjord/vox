use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Canal au sein d'un Espace
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Channel {
    pub id: Uuid,
    pub space_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: ChannelType,
    pub position: i32,
    pub is_ephemeral: bool,
    pub ephemeral_duration_hours: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "channel_type", rename_all = "lowercase")]
pub enum ChannelType {
    Text,
    Voice,
    Announcement,
    Thread,
}

impl Default for ChannelType {
    fn default() -> Self {
        Self::Text
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChannelRequest {
    #[validate(length(min = 1, max = 64, message = "Le nom du canal doit faire entre 1 et 64 caractères"))]
    pub name: String,
    #[validate(length(max = 256))]
    pub description: Option<String>,
    pub channel_type: Option<ChannelType>,
    pub is_ephemeral: Option<bool>,
    pub ephemeral_duration_hours: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateChannelRequest {
    #[validate(length(min = 1, max = 64))]
    pub name: Option<String>,
    #[validate(length(max = 256))]
    pub description: Option<String>,
    pub position: Option<i32>,
}
