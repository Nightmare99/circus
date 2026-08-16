use chrono::{DateTime, Utc};
use common::{Priority, TaskStatus};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Board {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Task {
    pub id: i64,
    pub board_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub assignee: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
