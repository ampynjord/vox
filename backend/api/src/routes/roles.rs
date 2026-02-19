use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
    routing::{get, post, put, delete},
};
use serde::Serialize;
use uuid::Uuid;
use chrono::Utc;
use validator::Validate;

use vox_shared::errors::VoxError;
use vox_shared::models::permission::{
    Role, RoleWithPermissions, CreateRoleRequest, UpdateRoleRequest, MemberRole, Permission,
};
use crate::middleware::AuthUser;
use crate::state::AppState;

/// Routes standalone : /roles/{id}, /permissions (montées sous /api/v1)
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/roles/:role_id", get(get_role).put(update_role).delete(delete_role))
        .route("/permissions", get(list_permissions))
}

/// Lister tous les rôles d'un espace
pub(super) async fn list_roles(
    AuthUser(_claims): AuthUser,
    State(state): State<AppState>,
    Path(space_id): Path<Uuid>,
) -> Result<Json<Vec<RoleWithPermissions>>, VoxError> {
    let roles = sqlx::query_as::<_, Role>(
        "SELECT * FROM roles WHERE space_id = $1 ORDER BY position ASC"
    )
    .bind(space_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?;

    let mut result = Vec::new();
    for role in roles {
        let permissions: Vec<String> = sqlx::query_scalar(
            "SELECT permission FROM role_permissions WHERE role_id = $1"
        )
        .bind(role.id)
        .fetch_all(&state.db)
        .await
        .map_err(|_| VoxError::InternalError)?;

        result.push(RoleWithPermissions {
            id: role.id,
            space_id: role.space_id,
            name: role.name,
            color: role.color,
            icon: role.icon,
            position: role.position,
            is_default: role.is_default,
            permissions,
        });
    }

    Ok(Json(result))
}

/// Créer un rôle
pub(super) async fn create_role(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
    Path(space_id): Path<Uuid>,
    Json(body): Json<CreateRoleRequest>,
) -> Result<(StatusCode, Json<RoleWithPermissions>), VoxError> {
    body.validate().map_err(|e| VoxError::BadRequest(e.to_string()))?;

    // Vérifier que l'utilisateur est owner ou a ManageRoles
    check_permission(&state, space_id, claims.sub, "manage_roles").await?;

    // Obtenir la position maximale actuelle
    let max_pos: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(position) FROM roles WHERE space_id = $1"
    )
    .bind(space_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?;

    let role_id = Uuid::new_v4();
    let now = Utc::now();
    let position = max_pos.unwrap_or(0) + 1;
    let is_default = body.is_default.unwrap_or(false);

    let role = sqlx::query_as::<_, Role>(
        r#"INSERT INTO roles (id, space_id, name, color, icon, position, is_default, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING *"#,
    )
    .bind(role_id)
    .bind(space_id)
    .bind(&body.name)
    .bind(&body.color)
    .bind(&body.icon)
    .bind(position)
    .bind(is_default)
    .bind(now)
    .bind(now)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Erreur création rôle: {}", e);
        VoxError::InternalError
    })?;

    // Ajouter les permissions
    for perm in &body.permissions {
        if Permission::from_str(perm).is_some() {
            sqlx::query(
                "INSERT INTO role_permissions (role_id, permission) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(role_id)
            .bind(perm)
            .execute(&state.db)
            .await
            .map_err(|_| VoxError::InternalError)?;
        }
    }

    tracing::info!("Rôle '{}' créé dans l'espace {}", body.name, space_id);

    Ok((StatusCode::CREATED, Json(RoleWithPermissions {
        id: role.id,
        space_id: role.space_id,
        name: role.name,
        color: role.color,
        icon: role.icon,
        position: role.position,
        is_default: role.is_default,
        permissions: body.permissions,
    })))
}

/// Obtenir un rôle avec ses permissions
async fn get_role(
    AuthUser(_claims): AuthUser,
    State(state): State<AppState>,
    Path(role_id): Path<Uuid>,
) -> Result<Json<RoleWithPermissions>, VoxError> {
    let role = sqlx::query_as::<_, Role>(
        "SELECT * FROM roles WHERE id = $1"
    )
    .bind(role_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?
    .ok_or(VoxError::NotFound)?;

    let permissions: Vec<String> = sqlx::query_scalar(
        "SELECT permission FROM role_permissions WHERE role_id = $1"
    )
    .bind(role_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?;

    Ok(Json(RoleWithPermissions {
        id: role.id,
        space_id: role.space_id,
        name: role.name,
        color: role.color,
        icon: role.icon,
        position: role.position,
        is_default: role.is_default,
        permissions,
    }))
}

/// Mettre à jour un rôle
async fn update_role(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
    Path(role_id): Path<Uuid>,
    Json(body): Json<UpdateRoleRequest>,
) -> Result<Json<RoleWithPermissions>, VoxError> {
    body.validate().map_err(|e| VoxError::BadRequest(e.to_string()))?;

    let role = sqlx::query_as::<_, Role>(
        "SELECT * FROM roles WHERE id = $1"
    )
    .bind(role_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?
    .ok_or(VoxError::NotFound)?;

    check_permission(&state, role.space_id, claims.sub, "manage_roles").await?;

    let now = Utc::now();
    let updated_name = body.name.as_deref().unwrap_or(&role.name);
    let updated_color = body.color.as_ref().or(role.color.as_ref());
    let updated_icon = body.icon.as_ref().or(role.icon.as_ref());
    let updated_position = body.position.unwrap_or(role.position);

    sqlx::query(
        r#"UPDATE roles SET name = $1, color = $2, icon = $3, position = $4, updated_at = $5
           WHERE id = $6"#,
    )
    .bind(updated_name)
    .bind(updated_color)
    .bind(updated_icon)
    .bind(updated_position)
    .bind(now)
    .bind(role_id)
    .execute(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?;

    // Mettre à jour les permissions si fournies
    let final_permissions = if let Some(ref perms) = body.permissions {
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(role_id)
            .execute(&state.db)
            .await
            .map_err(|_| VoxError::InternalError)?;

        for perm in perms {
            if Permission::from_str(perm).is_some() {
                sqlx::query(
                    "INSERT INTO role_permissions (role_id, permission) VALUES ($1, $2)"
                )
                .bind(role_id)
                .bind(perm)
                .execute(&state.db)
                .await
                .map_err(|_| VoxError::InternalError)?;
            }
        }
        perms.clone()
    } else {
        sqlx::query_scalar::<_, String>(
            "SELECT permission FROM role_permissions WHERE role_id = $1"
        )
        .bind(role_id)
        .fetch_all(&state.db)
        .await
        .map_err(|_| VoxError::InternalError)?
    };

    Ok(Json(RoleWithPermissions {
        id: role_id,
        space_id: role.space_id,
        name: updated_name.to_string(),
        color: updated_color.cloned(),
        icon: updated_icon.cloned(),
        position: updated_position,
        is_default: role.is_default,
        permissions: final_permissions,
    }))
}

/// Supprimer un rôle
async fn delete_role(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
    Path(role_id): Path<Uuid>,
) -> Result<StatusCode, VoxError> {
    let role = sqlx::query_as::<_, Role>(
        "SELECT * FROM roles WHERE id = $1"
    )
    .bind(role_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?
    .ok_or(VoxError::NotFound)?;

    check_permission(&state, role.space_id, claims.sub, "manage_roles").await?;

    // Ne pas autoriser la suppression du rôle par défaut
    if role.is_default {
        return Err(VoxError::BadRequest("Impossible de supprimer le rôle par défaut".into()));
    }

    // Supprimer les attributions, permissions et le rôle
    sqlx::query("DELETE FROM member_roles WHERE role_id = $1")
        .bind(role_id)
        .execute(&state.db)
        .await
        .map_err(|_| VoxError::InternalError)?;

    sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
        .bind(role_id)
        .execute(&state.db)
        .await
        .map_err(|_| VoxError::InternalError)?;

    sqlx::query("DELETE FROM channel_permission_overrides WHERE role_id = $1")
        .bind(role_id)
        .execute(&state.db)
        .await
        .map_err(|_| VoxError::InternalError)?;

    sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(role_id)
        .execute(&state.db)
        .await
        .map_err(|_| VoxError::InternalError)?;

    tracing::info!("Rôle '{}' supprimé", role.name);

    Ok(StatusCode::NO_CONTENT)
}

/// Assigner un rôle à un membre
pub(super) async fn assign_role(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
    Path((space_id, user_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, VoxError> {
    check_permission(&state, space_id, claims.sub, "manage_roles").await?;

    // Vérifier que le rôle appartient à cet espace
    let role_exists = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM roles WHERE id = $1 AND space_id = $2"
    )
    .bind(role_id)
    .bind(space_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?
    .is_some();

    if !role_exists {
        return Err(VoxError::NotFound);
    }

    // Vérifier que l'utilisateur est membre de l'espace
    let is_member = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM space_members WHERE space_id = $1 AND user_id = $2"
    )
    .bind(space_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?
    .is_some();

    if !is_member {
        return Err(VoxError::BadRequest("L'utilisateur n'est pas membre de cet espace".into()));
    }

    sqlx::query(
        r#"INSERT INTO member_roles (space_id, user_id, role_id, assigned_at)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (space_id, user_id, role_id) DO NOTHING"#,
    )
    .bind(space_id)
    .bind(user_id)
    .bind(role_id)
    .bind(Utc::now())
    .execute(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?;

    Ok(StatusCode::CREATED)
}

/// Révoquer un rôle d'un membre
pub(super) async fn revoke_role(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
    Path((space_id, user_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, VoxError> {
    check_permission(&state, space_id, claims.sub, "manage_roles").await?;

    sqlx::query(
        "DELETE FROM member_roles WHERE space_id = $1 AND user_id = $2 AND role_id = $3"
    )
    .bind(space_id)
    .bind(user_id)
    .bind(role_id)
    .execute(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Obtenir les rôles d'un membre
pub(super) async fn get_member_roles(
    AuthUser(_claims): AuthUser,
    State(state): State<AppState>,
    Path((space_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<RoleWithPermissions>>, VoxError> {
    let roles = sqlx::query_as::<_, Role>(
        r#"SELECT r.* FROM roles r
           INNER JOIN member_roles mr ON mr.role_id = r.id
           WHERE mr.space_id = $1 AND mr.user_id = $2
           ORDER BY r.position ASC"#,
    )
    .bind(space_id)
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?;

    let mut result = Vec::new();
    for role in roles {
        let permissions: Vec<String> = sqlx::query_scalar(
            "SELECT permission FROM role_permissions WHERE role_id = $1"
        )
        .bind(role.id)
        .fetch_all(&state.db)
        .await
        .map_err(|_| VoxError::InternalError)?;

        result.push(RoleWithPermissions {
            id: role.id,
            space_id: role.space_id,
            name: role.name,
            color: role.color,
            icon: role.icon,
            position: role.position,
            is_default: role.is_default,
            permissions,
        });
    }

    Ok(Json(result))
}

/// Lister toutes les permissions disponibles
#[derive(Serialize)]
struct PermissionInfo {
    key: String,
    category: String,
}

async fn list_permissions() -> Json<Vec<PermissionInfo>> {
    let permissions = vec![
        PermissionInfo { key: "manage_space".into(), category: "espace".into() },
        PermissionInfo { key: "manage_roles".into(), category: "espace".into() },
        PermissionInfo { key: "manage_channels".into(), category: "espace".into() },
        PermissionInfo { key: "kick_members".into(), category: "espace".into() },
        PermissionInfo { key: "ban_members".into(), category: "espace".into() },
        PermissionInfo { key: "invite_members".into(), category: "espace".into() },
        PermissionInfo { key: "view_channel".into(), category: "canal".into() },
        PermissionInfo { key: "send_messages".into(), category: "canal".into() },
        PermissionInfo { key: "manage_messages".into(), category: "canal".into() },
        PermissionInfo { key: "attach_files".into(), category: "canal".into() },
        PermissionInfo { key: "embed_links".into(), category: "canal".into() },
        PermissionInfo { key: "add_reactions".into(), category: "canal".into() },
        PermissionInfo { key: "mention_everyone".into(), category: "canal".into() },
        PermissionInfo { key: "voice_connect".into(), category: "vocal".into() },
        PermissionInfo { key: "voice_speak".into(), category: "vocal".into() },
        PermissionInfo { key: "voice_mute_members".into(), category: "vocal".into() },
        PermissionInfo { key: "voice_deafen_members".into(), category: "vocal".into() },
        PermissionInfo { key: "voice_move_members".into(), category: "vocal".into() },
        PermissionInfo { key: "screen_share".into(), category: "vocal".into() },
    ];
    Json(permissions)
}

// ========== Helpers ==========

/// Vérifie qu'un utilisateur a une permission donnée dans un espace
/// (Owner a toujours toutes les permissions)
async fn check_permission(
    state: &AppState,
    space_id: Uuid,
    user_id: Uuid,
    permission: &str,
) -> Result<(), VoxError> {
    // Vérifier si l'utilisateur est le propriétaire
    let is_owner = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM spaces WHERE id = $1 AND owner_id = $2"
    )
    .bind(space_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?
    .is_some();

    if is_owner {
        return Ok(());
    }

    // Vérifier les permissions via les rôles
    let has_permission = sqlx::query_scalar::<_, String>(
        r#"SELECT rp.permission FROM role_permissions rp
           INNER JOIN member_roles mr ON mr.role_id = rp.role_id
           WHERE mr.space_id = $1 AND mr.user_id = $2 AND rp.permission = $3
           LIMIT 1"#,
    )
    .bind(space_id)
    .bind(user_id)
    .bind(permission)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| VoxError::InternalError)?
    .is_some();

    // Vérifier aussi les permissions du rôle par défaut
    if !has_permission {
        let has_default = sqlx::query_scalar::<_, String>(
            r#"SELECT rp.permission FROM role_permissions rp
               INNER JOIN roles r ON r.id = rp.role_id
               WHERE r.space_id = $1 AND r.is_default = true AND rp.permission = $2
               LIMIT 1"#,
        )
        .bind(space_id)
        .bind(permission)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| VoxError::InternalError)?
        .is_some();

        if !has_default {
            return Err(VoxError::Forbidden);
        }
    }

    Ok(())
}
