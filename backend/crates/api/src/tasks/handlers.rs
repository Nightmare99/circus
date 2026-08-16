use crate::{
    auth::{ProjectScope, TaskScope},
    error::ApiError,
    serde_util::double_option,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use db::models::{AttachmentRow, CommentRow, TagRow, TaskRow};
use domain::{Action, Priority, TaskStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Default)]
pub struct TaskQuery {
    pub status: Option<TaskStatus>,
    pub assignee_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub search: Option<String>,
}

pub async fn list_tasks(
    State(state): State<AppState>,
    scope: ProjectScope,
    Query(q): Query<TaskQuery>,
) -> Result<Json<Vec<TaskRow>>, ApiError> {
    let filter = db::tasks::TaskFilter {
        status: q.status,
        assignee_id: q.assignee_id,
        tag_id: q.tag_id,
        search: q.search,
    };
    let tasks = db::tasks::list_for_project(&state.pool, scope.project_id, &filter).await?;
    Ok(Json(tasks))
}

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    #[serde(default = "default_priority")]
    pub priority: Priority,
    pub assignee_id: Option<Uuid>,
    pub due_date: Option<NaiveDate>,
}

fn default_priority() -> Priority {
    Priority::Medium
}

pub async fn create_task(
    State(state): State<AppState>,
    scope: ProjectScope,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<TaskRow>, ApiError> {
    if !(scope.is_superadmin || scope.role.can(Action::EditTask)) {
        return Err(ApiError::Forbidden);
    }
    let title = req.title.trim();
    if title.is_empty() {
        return Err(ApiError::BadRequest("title is required".into()));
    }
    if let Some(assignee_id) = req.assignee_id {
        require_project_member(&state, scope.project_id, assignee_id).await?;
    }

    let task_number = db::projects::next_task_number(&state.pool, scope.project_id).await?;
    let task = db::tasks::create(
        &state.pool,
        db::tasks::NewTask {
            org_id: scope.org_id,
            project_id: scope.project_id,
            task_number,
            title,
            description: req.description.as_deref(),
            priority: req.priority,
            assignee_id: req.assignee_id,
            reporter_id: scope.user_id,
            due_date: req.due_date,
        },
    )
    .await?;
    state.notify_project(scope.project_id);
    Ok(Json(task))
}

async fn require_project_member(
    state: &AppState,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<(), ApiError> {
    // Org owners/admins are implicit members of every project; only check
    // the explicit membership table when there isn't already one.
    if db::projects::find_membership(&state.pool, project_id, user_id)
        .await?
        .is_some()
    {
        return Ok(());
    }
    let Some(project) = db::projects::find_by_id(&state.pool, project_id).await? else {
        return Err(ApiError::NotFound);
    };
    match db::orgs::find_membership(&state.pool, project.org_id, user_id).await? {
        Some(m) if m.role >= domain::OrgRole::Admin => Ok(()),
        _ => Err(ApiError::BadRequest(
            "assignee must be a member of this project".into(),
        )),
    }
}

#[derive(Serialize)]
pub struct TaskDetail {
    #[serde(flatten)]
    pub task: TaskRow,
    pub tags: Vec<TagRow>,
    pub comments: Vec<CommentRow>,
    pub attachments: Vec<AttachmentRow>,
}

pub async fn get_task(
    State(state): State<AppState>,
    scope: TaskScope,
) -> Result<Json<TaskDetail>, ApiError> {
    let task = db::tasks::find_by_id(&state.pool, scope.task_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let tags = db::tags::list_for_task(&state.pool, scope.task_id).await?;
    let comments = db::comments::list_for_task(&state.pool, scope.task_id).await?;
    let attachments = db::attachments::list_for_task(&state.pool, scope.task_id).await?;
    Ok(Json(TaskDetail {
        task,
        tags,
        comments,
        attachments,
    }))
}

#[derive(Deserialize, Default)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub description: Option<Option<String>>,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    #[serde(default, deserialize_with = "double_option")]
    pub assignee_id: Option<Option<Uuid>>,
    #[serde(default, deserialize_with = "double_option")]
    pub due_date: Option<Option<NaiveDate>>,
}

pub async fn update_task(
    State(state): State<AppState>,
    scope: TaskScope,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<TaskRow>, ApiError> {
    if !(scope.is_superadmin || scope.role.can(Action::EditTask)) {
        return Err(ApiError::Forbidden);
    }
    if let Some(title) = &req.title {
        if title.trim().is_empty() {
            return Err(ApiError::BadRequest("title cannot be empty".into()));
        }
    }
    if let Some(Some(assignee_id)) = req.assignee_id {
        require_project_member(&state, scope.project_id, assignee_id).await?;
    }

    let task = db::tasks::update(
        &state.pool,
        scope.task_id,
        db::tasks::TaskUpdate {
            title: req.title.as_deref(),
            description: req.description.as_ref().map(|o| o.as_deref()),
            status: req.status,
            priority: req.priority,
            assignee_id: req.assignee_id,
            due_date: req.due_date,
        },
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    state.notify_project(scope.project_id);
    Ok(Json(task))
}

pub async fn delete_task(
    State(state): State<AppState>,
    scope: TaskScope,
) -> Result<StatusCode, ApiError> {
    if !(scope.is_superadmin || scope.role.can(Action::DeleteTask)) {
        return Err(ApiError::Forbidden);
    }
    let affected = db::tasks::delete(&state.pool, scope.task_id).await?;
    if affected == 0 {
        return Err(ApiError::NotFound);
    }
    state.notify_project(scope.project_id);
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct SetTagsRequest {
    pub tag_ids: Vec<Uuid>,
}

pub async fn set_task_tags(
    State(state): State<AppState>,
    scope: TaskScope,
    Json(req): Json<SetTagsRequest>,
) -> Result<Json<Vec<TagRow>>, ApiError> {
    if !(scope.is_superadmin || scope.role.can(Action::EditTask)) {
        return Err(ApiError::Forbidden);
    }
    let project_tags = db::tags::list_for_project(&state.pool, scope.project_id).await?;
    let valid_ids: std::collections::HashSet<Uuid> = project_tags.iter().map(|t| t.id).collect();
    if req.tag_ids.iter().any(|id| !valid_ids.contains(id)) {
        return Err(ApiError::BadRequest(
            "one or more tags do not belong to this project".into(),
        ));
    }
    db::tags::set_task_tags(&state.pool, scope.task_id, &req.tag_ids).await?;
    state.notify_project(scope.project_id);
    Ok(Json(
        db::tags::list_for_task(&state.pool, scope.task_id).await?,
    ))
}
