use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use uuid::Uuid;

use vox_shared::{
    errors::{VoxError, VoxResult},
    models::space::*,
    validation::validate_request,
};

use crate::middleware::AuthUser;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_space).get(list_my_spaces))
        .route("/:space_id", get(get_space).put(update_space).delete(delete_space))
        .route("/:space_id/join", post(join_space))
        .route("/:space_id/leave", post(leave_space))
        .route("/:space_id/members", get(list_members))
        // Routes rôles (intégrées ici pour éviter le conflit de nest)
        .route("/:space_id/roles", get(super::roles::list_roles).post(super::roles::create_role))
        .route("/:space_id/members/:user_id/roles/:role_id", post(super::roles::assign_role).delete(super::roles::revoke_role))
        .route("/:space_id/members/:user_id/roles", get(super::roles::get_member_roles))
}

/// POST /api/v1/spaces
async fn create_space(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateSpaceRequest>,
) -> VoxResult<Json<Space>> {
    validate_request(&req)?;

    let space = sqlx::query_as::<_, Space>(
        r#"
        INSERT INTO spaces (id, name, description, owner_id, is_public)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&req.name)
    .bind(&req.description)
    .bind(claims.sub)
    .bind(req.is_public.unwrap_or(true))
    .fetch_one(&state.db)
    .await?;

    // Ajouter le créateur comme membre Owner
    sqlx::query(
        "INSERT INTO space_members (space_id, user_id, role) VALUES ($1, $2, 'owner')"
    )
    .bind(space.id)
    .bind(claims.sub)
    .execute(&state.db)
    .await?;

    Ok(Json(space))
}

/// GET /api/v1/spaces
async fn list_my_spaces(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> VoxResult<Json<Vec<Space>>> {
    let spaces = sqlx::query_as::<_, Space>(
        r#"
        SELECT s.* FROM spaces s
        INNER JOIN space_members sm ON s.id = sm.space_id
        WHERE sm.user_id = $1
        ORDER BY s.name
        "#
    )
    .bind(claims.sub)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(spaces))
}

/// GET /api/v1/spaces/:space_id
async fn get_space(
    State(state): State<AppState>,
    AuthUser(_claims): AuthUser,
    Path(space_id): Path<Uuid>,
) -> VoxResult<Json<Space>> {
    let space = sqlx::query_as::<_, Space>(
        "SELECT * FROM spaces WHERE id = $1"
    )
    .bind(space_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::NotFound)?;

    Ok(Json(space))
}

/// PUT /api/v1/spaces/:space_id
async fn update_space(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(space_id): Path<Uuid>,
    Json(req): Json<UpdateSpaceRequest>,
) -> VoxResult<Json<Space>> {
    validate_request(&req)?;

    // Vérifier que l'utilisateur est owner ou admin
    let member = sqlx::query_as::<_, SpaceMember>(
        "SELECT * FROM space_members WHERE space_id = $1 AND user_id = $2"
    )
    .bind(space_id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::Forbidden)?;

    if member.role != SpaceRole::Owner && member.role != SpaceRole::Admin {
        return Err(VoxError::Forbidden);
    }

    let space = sqlx::query_as::<_, Space>(
        r#"
        UPDATE spaces SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            is_public = COALESCE($4, is_public)
        WHERE id = $1
        RETURNING *
        "#
    )
    .bind(space_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(req.is_public)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(space))
}

/// DELETE /api/v1/spaces/:space_id
async fn delete_space(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(space_id): Path<Uuid>,
) -> VoxResult<Json<serde_json::Value>> {
    // Seul l'owner peut supprimer
    let space = sqlx::query_as::<_, Space>(
        "SELECT * FROM spaces WHERE id = $1"
    )
    .bind(space_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::NotFound)?;

    if space.owner_id != claims.sub {
        return Err(VoxError::Forbidden);
    }

    sqlx::query("DELETE FROM spaces WHERE id = $1")
        .bind(space_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"message": "Espace supprimé"})))
}

/// POST /api/v1/spaces/:space_id/join
async fn join_space(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(space_id): Path<Uuid>,
) -> VoxResult<Json<SpaceMember>> {
    // Vérifier que l'espace existe et est public
    let space = sqlx::query_as::<_, Space>(
        "SELECT * FROM spaces WHERE id = $1"
    )
    .bind(space_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::NotFound)?;

    if !space.is_public {
        return Err(VoxError::Forbidden);
    }

    let member = sqlx::query_as::<_, SpaceMember>(
        r#"
        INSERT INTO space_members (space_id, user_id, role)
        VALUES ($1, $2, 'member')
        ON CONFLICT (space_id, user_id) DO NOTHING
        RETURNING *
        "#
    )
    .bind(space_id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::Conflict("Vous êtes déjà membre de cet espace".to_string()))?;

    // Incrémenter le compteur de membres
    sqlx::query("UPDATE spaces SET member_count = member_count + 1 WHERE id = $1")
        .bind(space_id)
        .execute(&state.db)
        .await?;

    Ok(Json(member))
}

/// POST /api/v1/spaces/:space_id/leave
async fn leave_space(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(space_id): Path<Uuid>,
) -> VoxResult<Json<serde_json::Value>> {
    // L'owner ne peut pas quitter
    let member = sqlx::query_as::<_, SpaceMember>(
        "SELECT * FROM space_members WHERE space_id = $1 AND user_id = $2"
    )
    .bind(space_id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(VoxError::NotFound)?;

    if member.role == SpaceRole::Owner {
        return Err(VoxError::BadRequest("L'owner ne peut pas quitter l'espace. Transférez la propriété d'abord.".to_string()));
    }

    sqlx::query("DELETE FROM space_members WHERE space_id = $1 AND user_id = $2")
        .bind(space_id)
        .bind(claims.sub)
        .execute(&state.db)
        .await?;

    sqlx::query("UPDATE spaces SET member_count = GREATEST(member_count - 1, 0) WHERE id = $1")
        .bind(space_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"message": "Vous avez quitté l'espace"})))
}

/// GET /api/v1/spaces/:space_id/members
async fn list_members(
    State(state): State<AppState>,
    AuthUser(_claims): AuthUser,
    Path(space_id): Path<Uuid>,
) -> VoxResult<Json<Vec<SpaceMember>>> {
    let members = sqlx::query_as::<_, SpaceMember>(
        "SELECT * FROM space_members WHERE space_id = $1 ORDER BY joined_at"
    )
    .bind(space_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(members))
}
