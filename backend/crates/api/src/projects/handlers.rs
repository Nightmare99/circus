use crate::{
    auth::{OrgScope, ProjectScope},
    error::ApiError,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use db::models::{ProjectMemberRow, ProjectMembershipRow, ProjectRow, TagRow};
use domain::{Action, OrgRole, ProjectRole};
use serde::Deserialize;
use uuid::Uuid;

fn normalize_key(raw: &str) -> Result<String, ApiError> {
    let key = raw.trim().to_uppercase();
    let valid = (2..=10).contains(&key.len())
        && key.chars().all(|c| c.is_ascii_alphanumeric())
        && key.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
    if !valid {
        return Err(ApiError::BadRequest(
            "key must be 2-10 letters/digits, starting with a letter".into(),
        ));
    }
    Ok(key)
}

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_project(
    State(state): State<AppState>,
    scope: OrgScope,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<ProjectRow>, ApiError> {
    if !(scope.is_superadmin || scope.role.can(Action::CreateProject)) {
        return Err(ApiError::Forbidden);
    }
    let name = req.name.trim();
    if name.is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    let key = normalize_key(&req.key)?;
    let project = db::projects::create(
        &state.pool,
        scope.org_id,
        &key,
        name,
        req.description.as_deref(),
    )
    .await?;
    Ok(Json(project))
}

pub async fn list_projects(
    State(state): State<AppState>,
    scope: OrgScope,
) -> Result<Json<Vec<ProjectRow>>, ApiError> {
    let is_admin = scope.is_superadmin || scope.role >= OrgRole::Admin;
    let projects =
        db::projects::list_visible_for_user(&state.pool, scope.org_id, scope.user_id, is_admin)
            .await?;
    Ok(Json(projects))
}

pub async fn get_project(
    State(state): State<AppState>,
    scope: ProjectScope,
) -> Result<Json<ProjectRow>, ApiError> {
    let project = db::projects::find_by_id(&state.pool, scope.project_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(project))
}

pub async fn delete_project(
    State(state): State<AppState>,
    scope: ProjectScope,
) -> Result<StatusCode, ApiError> {
    if !(scope.is_superadmin || scope.role.can(Action::DeleteProject)) {
        return Err(ApiError::Forbidden);
    }
    let affected = db::projects::delete(&state.pool, scope.project_id).await?;
    if affected == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_members(
    State(state): State<AppState>,
    scope: ProjectScope,
) -> Result<Json<Vec<ProjectMemberRow>>, ApiError> {
    Ok(Json(
        db::projects::list_members(&state.pool, scope.project_id).await?,
    ))
}

#[derive(Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub role: ProjectRole,
}

pub async fn add_member(
    State(state): State<AppState>,
    scope: ProjectScope,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<ProjectMembershipRow>, ApiError> {
    if !(scope.is_superadmin || scope.role.can(Action::ManageProjectMembers)) {
        return Err(ApiError::Forbidden);
    }
    if db::orgs::find_membership(&state.pool, scope.org_id, req.user_id)
        .await?
        .is_none()
    {
        return Err(ApiError::BadRequest(
            "user is not a member of this organization".into(),
        ));
    }
    let membership =
        db::projects::add_member(&state.pool, scope.project_id, req.user_id, req.role).await?;
    Ok(Json(membership))
}

pub async fn remove_member(
    State(state): State<AppState>,
    scope: ProjectScope,
    Path((_, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    if !(scope.is_superadmin || scope.role.can(Action::ManageProjectMembers)) {
        return Err(ApiError::Forbidden);
    }
    let affected = db::projects::remove_member(&state.pool, scope.project_id, user_id).await?;
    if affected == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    #[serde(default = "default_tag_color")]
    pub color: String,
}

fn default_tag_color() -> String {
    "#6b7280".to_string()
}

pub async fn list_tags(
    State(state): State<AppState>,
    scope: ProjectScope,
) -> Result<Json<Vec<TagRow>>, ApiError> {
    Ok(Json(
        db::tags::list_for_project(&state.pool, scope.project_id).await?,
    ))
}

pub async fn create_tag(
    State(state): State<AppState>,
    scope: ProjectScope,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<TagRow>, ApiError> {
    if !(scope.is_superadmin || scope.role.can(Action::EditTask)) {
        return Err(ApiError::Forbidden);
    }
    let name = req.name.trim();
    if name.is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    let tag = db::tags::create(&state.pool, scope.project_id, name, &req.color).await?;
    Ok(Json(tag))
}

pub async fn delete_tag(
    State(state): State<AppState>,
    scope: ProjectScope,
    Path((_, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    if !(scope.is_superadmin || scope.role.can(Action::ManageProjectMembers)) {
        return Err(ApiError::Forbidden);
    }
    let affected = db::tags::delete(&state.pool, scope.project_id, tag_id).await?;
    if affected == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
