use chrono::{DateTime, NaiveDate, Utc};
use domain::{InstanceRole, OrgRole, Priority, ProjectRole, TaskStatus};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: String,
    pub instance_role: InstanceRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct OrgRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct OrgMembershipRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub role: OrgRole,
    pub created_at: DateTime<Utc>,
}

/// An org membership joined with the user's public profile fields.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct OrgMemberRow {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: OrgRole,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct InviteRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub email: String,
    pub role: OrgRole,
    #[serde(skip_serializing)]
    pub token: String,
    pub invited_by: Uuid,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ProjectRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ProjectMembershipRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub role: ProjectRole,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ProjectMemberRow {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: ProjectRole,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct TagRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct TaskRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub task_number: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub assignee_id: Option<Uuid>,
    pub reporter_id: Uuid,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct CommentRow {
    pub id: Uuid,
    pub task_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct AttachmentRow {
    pub id: Uuid,
    pub task_id: Uuid,
    pub uploaded_by: Uuid,
    pub file_name: String,
    pub content_type: String,
    pub size_bytes: i64,
    #[serde(skip_serializing)]
    pub storage_key: String,
    pub created_at: DateTime<Utc>,
}
