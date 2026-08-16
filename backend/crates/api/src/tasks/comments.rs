use crate::{auth::TaskScope, error::ApiError, state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use db::models::CommentRow;
use domain::Action;
use serde::Deserialize;
use uuid::Uuid;

pub async fn list_comments(
    State(state): State<AppState>,
    scope: TaskScope,
) -> Result<Json<Vec<CommentRow>>, ApiError> {
    Ok(Json(
        db::comments::list_for_task(&state.pool, scope.task_id).await?,
    ))
}

#[derive(Deserialize)]
pub struct CreateCommentRequest {
    pub body: String,
}

pub async fn create_comment(
    State(state): State<AppState>,
    scope: TaskScope,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<CommentRow>, ApiError> {
    if !(scope.is_superadmin || scope.role.can(Action::CommentOnTask)) {
        return Err(ApiError::Forbidden);
    }
    let body = req.body.trim();
    if body.is_empty() {
        return Err(ApiError::BadRequest("comment body is required".into()));
    }
    let comment = db::comments::create(&state.pool, scope.task_id, scope.user_id, body).await?;
    Ok(Json(comment))
}

async fn require_author_or_lead(
    state: &AppState,
    scope: &TaskScope,
    comment_id: Uuid,
) -> Result<(), ApiError> {
    let comment = db::comments::find_by_id(&state.pool, comment_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if comment.task_id != scope.task_id {
        return Err(ApiError::NotFound);
    }
    let is_author = comment.author_id == scope.user_id;
    let is_lead = scope.is_superadmin || scope.role.can(Action::ManageProjectMembers);
    if !(is_author || is_lead) {
        return Err(ApiError::Forbidden);
    }
    let _ = state;
    Ok(())
}

#[derive(Deserialize)]
pub struct UpdateCommentRequest {
    pub body: String,
}

pub async fn update_comment(
    State(state): State<AppState>,
    scope: TaskScope,
    Path((_, comment_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateCommentRequest>,
) -> Result<Json<CommentRow>, ApiError> {
    require_author_or_lead(&state, &scope, comment_id).await?;
    let body = req.body.trim();
    if body.is_empty() {
        return Err(ApiError::BadRequest("comment body is required".into()));
    }
    let comment = db::comments::update_body(&state.pool, comment_id, body)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(comment))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    scope: TaskScope,
    Path((_, comment_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    require_author_or_lead(&state, &scope, comment_id).await?;
    db::comments::delete(&state.pool, comment_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
