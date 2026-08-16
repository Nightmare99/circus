use crate::{auth::TaskScope, error::ApiError, state::AppState, storage::Storage};
use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use db::models::AttachmentRow;
use domain::Action;
use uuid::Uuid;

pub async fn list_attachments(
    State(state): State<AppState>,
    scope: TaskScope,
) -> Result<Json<Vec<AttachmentRow>>, ApiError> {
    Ok(Json(
        db::attachments::list_for_task(&state.pool, scope.task_id).await?,
    ))
}

pub async fn upload_attachment(
    State(state): State<AppState>,
    scope: TaskScope,
    mut multipart: Multipart,
) -> Result<Json<AttachmentRow>, ApiError> {
    if !(scope.is_superadmin || scope.role.can(Action::UploadAttachment)) {
        return Err(ApiError::Forbidden);
    }

    let mut uploaded = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        if field.name() != Some("file") {
            continue;
        }
        let file_name = field.file_name().unwrap_or("file").to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
        uploaded = Some((file_name, content_type, data));
        break;
    }
    let (file_name, content_type, data) =
        uploaded.ok_or_else(|| ApiError::BadRequest("missing \"file\" field".into()))?;

    let max_bytes = state.max_upload_mb * 1024 * 1024;
    if data.len() > max_bytes {
        return Err(ApiError::BadRequest(format!(
            "file exceeds the {} MB limit",
            state.max_upload_mb
        )));
    }

    let attachment_id = Uuid::new_v4();
    let key = Storage::key_for(scope.task_id, attachment_id, &file_name);
    state
        .storage
        .write(&key, &data)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    let attachment = db::attachments::create(
        &state.pool,
        db::attachments::NewAttachment {
            id: attachment_id,
            task_id: scope.task_id,
            uploaded_by: scope.user_id,
            file_name: &file_name,
            content_type: &content_type,
            size_bytes: data.len() as i64,
            storage_key: &key,
        },
    )
    .await?;
    state.notify_project(scope.project_id);
    Ok(Json(attachment))
}

pub async fn download_attachment(
    State(state): State<AppState>,
    scope: TaskScope,
    Path((_, attachment_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, ApiError> {
    let attachment = db::attachments::find_by_id(&state.pool, attachment_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if attachment.task_id != scope.task_id {
        return Err(ApiError::NotFound);
    }
    let data = state
        .storage
        .read(&attachment.storage_key)
        .await
        .map_err(|_| ApiError::NotFound)?;

    let mut response = Response::new(Body::from(data));
    if let Ok(v) = HeaderValue::from_str(&attachment.content_type) {
        response.headers_mut().insert(header::CONTENT_TYPE, v);
    }
    if let Ok(v) = HeaderValue::from_str(&format!(
        "attachment; filename=\"{}\"",
        attachment.file_name.replace('"', "")
    )) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, v);
    }
    Ok(response)
}

pub async fn delete_attachment(
    State(state): State<AppState>,
    scope: TaskScope,
    Path((_, attachment_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let attachment = db::attachments::find_by_id(&state.pool, attachment_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if attachment.task_id != scope.task_id {
        return Err(ApiError::NotFound);
    }
    let is_uploader = attachment.uploaded_by == scope.user_id;
    let is_lead = scope.is_superadmin || scope.role.can(Action::ManageProjectMembers);
    if !(is_uploader || is_lead) {
        return Err(ApiError::Forbidden);
    }
    db::attachments::delete(&state.pool, attachment_id).await?;
    let _ = state.storage.delete(&attachment.storage_key).await;
    state.notify_project(scope.project_id);
    Ok(StatusCode::NO_CONTENT)
}
