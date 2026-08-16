use crate::{
    auth::{handlers::UserPublic, AuthUser},
    error::ApiError,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use db::models::OrgRow;
use domain::InstanceRole;
use serde::Deserialize;
use uuid::Uuid;

fn require_superadmin(auth: &AuthUser) -> Result<(), ApiError> {
    if auth.is_superadmin() {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

pub async fn list_orgs(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<OrgRow>>, ApiError> {
    require_superadmin(&auth)?;
    Ok(Json(db::orgs::list_all(&state.pool).await?))
}

pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<UserPublic>>, ApiError> {
    require_superadmin(&auth)?;
    let users = db::users::list_all(&state.pool).await?;
    Ok(Json(users.into_iter().map(UserPublic::from).collect()))
}

#[derive(Deserialize)]
pub struct UpdateInstanceRoleRequest {
    pub instance_role: InstanceRole,
}

pub async fn update_user_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateInstanceRoleRequest>,
) -> Result<Json<UserPublic>, ApiError> {
    require_superadmin(&auth)?;
    if user_id == auth.user_id && req.instance_role != InstanceRole::Superadmin {
        return Err(ApiError::Conflict(
            "cannot remove your own superadmin role".into(),
        ));
    }
    let user = db::users::update_instance_role(&state.pool, user_id, req.instance_role)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(user.into()))
}
