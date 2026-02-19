use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, Error)]
pub enum VoxError {
    #[error("Ressource non trouvée")]
    NotFound,

    #[error("Non autorisé")]
    Unauthorized,

    #[error("Accès interdit")]
    Forbidden,

    #[error("Données invalides: {0}")]
    BadRequest(String),

    #[error("Conflit: {0}")]
    Conflict(String),

    #[error("Erreur interne du serveur")]
    InternalError,

    #[error("Erreur de base de données: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Erreur de validation: {0}")]
    ValidationError(String),
}

impl IntoResponse for VoxError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            VoxError::NotFound => (StatusCode::NOT_FOUND, "Ressource non trouvée".to_string()),
            VoxError::Unauthorized => (StatusCode::UNAUTHORIZED, "Non autorisé".to_string()),
            VoxError::Forbidden => (StatusCode::FORBIDDEN, "Accès interdit".to_string()),
            VoxError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            VoxError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            VoxError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Erreur interne".to_string()),
            VoxError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Erreur de base de données".to_string()),
            VoxError::ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
        };

        let body = json!({
            "error": {
                "code": status.as_u16(),
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

pub type VoxResult<T> = Result<T, VoxError>;
