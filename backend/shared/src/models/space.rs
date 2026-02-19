use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Un "Espace" dans Vox (équivalent d'un serveur, mais en mieux)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Space {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub owner_id: Uuid,
    pub is_public: bool,
    pub member_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSpaceRequest {
    #[validate(length(min = 2, max = 64, message = "Le nom de l'espace doit faire entre 2 et 64 caractères"))]
    pub name: String,
    #[validate(length(max = 512))]
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSpaceRequest {
    #[validate(length(min = 2, max = 64))]
    pub name: Option<String>,
    #[validate(length(max = 512))]
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

/// Membre d'un espace
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SpaceMember {
    pub space_id: Uuid,
    pub user_id: Uuid,
    pub role: SpaceRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "space_role", rename_all = "lowercase")]
pub enum SpaceRole {
    Owner,
    Admin,
    Moderator,
    Member,
}

impl Default for SpaceRole {
    fn default() -> Self {
        Self::Member
    }
}
