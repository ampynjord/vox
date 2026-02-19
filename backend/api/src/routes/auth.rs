use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use uuid::Uuid;

use vox_shared::{
    auth::{create_token, hash_password, verify_password},
    errors::{VoxError, VoxResult},
    models::user::*,
    validation::validate_request,
};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

/// POST /api/v1/auth/register
async fn register(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> VoxResult<Json<AuthResponse>> {
    validate_request(&req)?;

    // Vérifier si l'utilisateur existe déjà
    let existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 OR username = $2)"
    )
    .bind(&req.email)
    .bind(&req.username)
    .fetch_one(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?;

    if existing {
        return Err(VoxError::Conflict("Un utilisateur avec cet email ou ce nom existe déjà".to_string()));
    }

    // Hasher le mot de passe
    let password_hash = hash_password(&req.password)?;

    // Créer l'utilisateur
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, display_name, email, password_hash)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&req.username)
    .bind(&req.display_name)
    .bind(&req.email)
    .bind(&password_hash)
    .fetch_one(&state.db)
    .await?;

    // Générer le token
    let token = create_token(user.id, &user.username, &state.jwt_secret, state.jwt_expiration_hours)?;

    Ok(Json(AuthResponse {
        token,
        user: UserPublic::from(user),
    }))
}

/// POST /api/v1/auth/login
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> VoxResult<Json<AuthResponse>> {
    validate_request(&req)?;

    // Chercher l'utilisateur par email
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&req.email)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::Unauthorized)?;

    // Vérifier le mot de passe
    let valid = verify_password(&req.password, &user.password_hash)?;
    if !valid {
        return Err(VoxError::Unauthorized);
    }

    // Générer le token
    let token = create_token(user.id, &user.username, &state.jwt_secret, state.jwt_expiration_hours)?;

    Ok(Json(AuthResponse {
        token,
        user: UserPublic::from(user),
    }))
}
