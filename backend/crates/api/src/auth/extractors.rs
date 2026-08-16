use super::jwt::{self, TokenType};
use crate::{error::ApiError, state::AppState};
use axum::{
    extract::{FromRequestParts, Path},
    http::{header, request::Parts},
};
use domain::{InstanceRole, OrgRole, ProjectRole};
use std::collections::HashMap;
use uuid::Uuid;

/// An authenticated user, resolved from a valid `Authorization: Bearer` access token.
/// Carries no org/project context — see [`OrgScope`], [`ProjectScope`], [`TaskScope`].
#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub instance_role: InstanceRole,
}

impl AuthUser {
    pub fn is_superadmin(&self) -> bool {
        self.instance_role == InstanceRole::Superadmin
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;
        let token = header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized)?;
        let claims =
            jwt::decode_token(token, &state.jwt_secret).map_err(|_| ApiError::Unauthorized)?;
        if claims.typ != TokenType::Access {
            return Err(ApiError::Unauthorized);
        }
        let instance_role = claims.role.ok_or(ApiError::Unauthorized)?;
        Ok(AuthUser {
            user_id: claims.sub,
            instance_role,
        })
    }
}

async fn path_uuid(parts: &mut Parts, state: &AppState, key: &str) -> Result<Uuid, ApiError> {
    let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
        .await
        .map_err(|_| ApiError::BadRequest("invalid path".into()))?;
    params
        .get(key)
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or_else(|| ApiError::BadRequest(format!("missing path param: {key}")))
}

/// A user's resolved access to an org (`{org_id}` path param). Instance
/// superadmins are treated as an implicit `Owner` of every org.
pub struct OrgScope {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub role: OrgRole,
    pub is_superadmin: bool,
}

impl FromRequestParts<AppState> for OrgScope {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = AuthUser::from_request_parts(parts, state).await?;
        let org_id = path_uuid(parts, state, "org_id").await?;
        let is_superadmin = auth.is_superadmin();
        let role = if is_superadmin {
            OrgRole::Owner
        } else {
            db::orgs::find_membership(&state.pool, org_id, auth.user_id)
                .await?
                .map(|m| m.role)
                .ok_or(ApiError::Forbidden)?
        };
        Ok(OrgScope {
            user_id: auth.user_id,
            org_id,
            role,
            is_superadmin,
        })
    }
}

/// Resolve a user's effective role on a project: org owners/admins get an
/// implicit `Lead`, everyone else needs an explicit project membership row.
pub async fn resolve_project_role(
    state: &AppState,
    org_id: Uuid,
    project_id: Uuid,
    user_id: Uuid,
    is_superadmin: bool,
) -> Result<ProjectRole, ApiError> {
    if is_superadmin {
        return Ok(ProjectRole::Lead);
    }
    let org_membership = db::orgs::find_membership(&state.pool, org_id, user_id)
        .await?
        .ok_or(ApiError::Forbidden)?;
    if org_membership.role >= OrgRole::Admin {
        return Ok(ProjectRole::Lead);
    }
    db::projects::find_membership(&state.pool, project_id, user_id)
        .await?
        .map(|m| m.role)
        .ok_or(ApiError::Forbidden)
}

/// A user's resolved access to a project (`{project_id}` path param).
pub struct ProjectScope {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub role: ProjectRole,
    pub is_superadmin: bool,
}

impl FromRequestParts<AppState> for ProjectScope {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = AuthUser::from_request_parts(parts, state).await?;
        let project_id = path_uuid(parts, state, "project_id").await?;
        let project = db::projects::find_by_id(&state.pool, project_id)
            .await?
            .ok_or(ApiError::NotFound)?;
        let is_superadmin = auth.is_superadmin();
        let role = resolve_project_role(
            state,
            project.org_id,
            project_id,
            auth.user_id,
            is_superadmin,
        )
        .await?;
        Ok(ProjectScope {
            user_id: auth.user_id,
            org_id: project.org_id,
            project_id,
            role,
            is_superadmin,
        })
    }
}

/// A user's resolved access to a task (`{task_id}` path param), derived
/// through the task's project — used for routes not nested under a project.
pub struct TaskScope {
    pub user_id: Uuid,
    // Kept for parity with ProjectScope / future org-scoped checks (e.g. audit logging).
    #[allow(dead_code)]
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub task_id: Uuid,
    pub role: ProjectRole,
    pub is_superadmin: bool,
}

impl FromRequestParts<AppState> for TaskScope {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = AuthUser::from_request_parts(parts, state).await?;
        let task_id = path_uuid(parts, state, "task_id").await?;
        let task = db::tasks::find_by_id(&state.pool, task_id)
            .await?
            .ok_or(ApiError::NotFound)?;
        let is_superadmin = auth.is_superadmin();
        let role = resolve_project_role(
            state,
            task.org_id,
            task.project_id,
            auth.user_id,
            is_superadmin,
        )
        .await?;
        Ok(TaskScope {
            user_id: auth.user_id,
            org_id: task.org_id,
            project_id: task.project_id,
            task_id,
            role,
            is_superadmin,
        })
    }
}
