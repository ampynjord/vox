use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};
use vox_shared::auth::{validate_token, Claims};
use vox_shared::errors::VoxError;
use crate::state::AppState;

/// Extracteur d'authentification — vérifie le token JWT
pub struct AuthUser(pub Claims);

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = VoxError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        let token = extract_bearer_token(headers)?;
        let claims = validate_token(&token, &state.jwt_secret)?;
        Ok(AuthUser(claims))
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<String, VoxError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(VoxError::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(VoxError::Unauthorized);
    }

    Ok(auth_header[7..].to_string())
}
