use crate::{
    auth::{AuthUser, OrgScope},
    error::ApiError,
    state::AppState,
    util,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use db::models::{InviteRow, OrgMemberRow, OrgMembershipRow, OrgRow};
use domain::{Action, OrgRole};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
}

pub async fn create_org(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateOrgRequest>,
) -> Result<Json<OrgRow>, ApiError> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    let slug = util::unique_org_slug(&state.pool, name).await?;
    let org = db::orgs::create(&state.pool, name, &slug).await?;
    db::orgs::add_member(&state.pool, org.id, auth.user_id, OrgRole::Owner).await?;
    Ok(Json(org))
}

pub async fn list_orgs(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<OrgRow>>, ApiError> {
    Ok(Json(
        db::orgs::list_for_user(&state.pool, auth.user_id).await?,
    ))
}

pub async fn get_org(
    State(state): State<AppState>,
    scope: OrgScope,
) -> Result<Json<OrgRow>, ApiError> {
    let org = db::orgs::find_by_id(&state.pool, scope.org_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(org))
}

pub async fn list_members(
    State(state): State<AppState>,
    scope: OrgScope,
) -> Result<Json<Vec<OrgMemberRow>>, ApiError> {
    Ok(Json(
        db::orgs::list_members(&state.pool, scope.org_id).await?,
    ))
}

fn require_manage_members(scope: &OrgScope) -> Result<(), ApiError> {
    if scope.is_superadmin || scope.role.can(Action::ManageOrgMembers) {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

#[derive(Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: OrgRole,
}

pub async fn update_member_role(
    State(state): State<AppState>,
    scope: OrgScope,
    Path((_, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> Result<Json<OrgMembershipRow>, ApiError> {
    require_manage_members(&scope)?;
    if req.role != OrgRole::Owner {
        guard_not_last_owner(&state, scope.org_id, user_id).await?;
    }
    let updated = db::orgs::update_member_role(&state.pool, scope.org_id, user_id, req.role)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(updated))
}

pub async fn remove_member(
    State(state): State<AppState>,
    scope: OrgScope,
    Path((_, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    require_manage_members(&scope)?;
    guard_not_last_owner(&state, scope.org_id, user_id).await?;
    let affected = db::orgs::remove_member(&state.pool, scope.org_id, user_id).await?;
    if affected == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn guard_not_last_owner(
    state: &AppState,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<(), ApiError> {
    let Some(membership) = db::orgs::find_membership(&state.pool, org_id, user_id).await? else {
        return Ok(());
    };
    if membership.role == OrgRole::Owner {
        let owners = db::orgs::count_owners(&state.pool, org_id).await?;
        if owners <= 1 {
            return Err(ApiError::Conflict(
                "cannot remove or demote the last owner".into(),
            ));
        }
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct CreateInviteRequest {
    pub email: String,
    pub role: OrgRole,
}

#[derive(Serialize)]
pub struct InviteResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub email: String,
    pub role: OrgRole,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

pub async fn create_invite(
    State(state): State<AppState>,
    scope: OrgScope,
    Json(req): Json<CreateInviteRequest>,
) -> Result<Json<InviteResponse>, ApiError> {
    require_manage_members(&scope)?;
    let email = req.email.trim().to_lowercase();
    if !email.contains('@') {
        return Err(ApiError::BadRequest("invalid email".into()));
    }
    let token = util::random_token();
    let expires_at = Utc::now() + Duration::days(7);
    let invite = db::invites::create(
        &state.pool,
        scope.org_id,
        &email,
        req.role,
        &token,
        scope.user_id,
        expires_at,
    )
    .await?;
    Ok(Json(InviteResponse {
        id: invite.id,
        org_id: invite.org_id,
        email: invite.email,
        role: invite.role,
        token,
        expires_at: invite.expires_at,
    }))
}

pub async fn list_invites(
    State(state): State<AppState>,
    scope: OrgScope,
) -> Result<Json<Vec<InviteRow>>, ApiError> {
    require_manage_members(&scope)?;
    Ok(Json(
        db::invites::list_pending(&state.pool, scope.org_id).await?,
    ))
}

pub async fn revoke_invite(
    State(state): State<AppState>,
    scope: OrgScope,
    Path((_, invite_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    require_manage_members(&scope)?;
    let affected = db::invites::revoke(&state.pool, scope.org_id, invite_id).await?;
    if affected == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct AcceptInviteRequest {
    pub token: String,
}

pub async fn accept_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<AcceptInviteRequest>,
) -> Result<Json<OrgRow>, ApiError> {
    let invite = db::invites::find_by_token(&state.pool, &req.token)
        .await?
        .ok_or(ApiError::NotFound)?;
    if invite.accepted_at.is_some() {
        return Err(ApiError::Conflict("invite already used".into()));
    }
    if invite.expires_at < Utc::now() {
        return Err(ApiError::Conflict("invite expired".into()));
    }
    let user = db::users::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    if user.email.to_lowercase() != invite.email.to_lowercase() {
        return Err(ApiError::Forbidden);
    }
    if db::orgs::find_membership(&state.pool, invite.org_id, auth.user_id)
        .await?
        .is_none()
    {
        db::orgs::add_member(&state.pool, invite.org_id, auth.user_id, invite.role).await?;
    }
    db::invites::mark_accepted(&state.pool, invite.id).await?;
    let org = db::orgs::find_by_id(&state.pool, invite.org_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(org))
}
