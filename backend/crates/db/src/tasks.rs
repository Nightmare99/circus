use crate::models::TaskRow;
use chrono::NaiveDate;
use domain::{Priority, TaskStatus};
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

pub struct NewTask<'a> {
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub task_number: i64,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub priority: Priority,
    pub assignee_id: Option<Uuid>,
    pub reporter_id: Uuid,
    pub due_date: Option<NaiveDate>,
}

pub async fn create(pool: &PgPool, t: NewTask<'_>) -> Result<TaskRow, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(
        r#"INSERT INTO tasks
             (org_id, project_id, task_number, title, description, priority, assignee_id, reporter_id, due_date)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING id, org_id, project_id, task_number, title, description, status, priority,
                     assignee_id, reporter_id, due_date, created_at, updated_at"#,
    )
    .bind(t.org_id)
    .bind(t.project_id)
    .bind(t.task_number)
    .bind(t.title)
    .bind(t.description)
    .bind(t.priority)
    .bind(t.assignee_id)
    .bind(t.reporter_id)
    .bind(t.due_date)
    .fetch_one(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<TaskRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(
        r#"SELECT id, org_id, project_id, task_number, title, description, status, priority,
                  assignee_id, reporter_id, due_date, created_at, updated_at
           FROM tasks WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

#[derive(Default)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub assignee_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    /// case-insensitive substring match against title
    pub search: Option<String>,
}

pub async fn list_for_project(
    pool: &PgPool,
    project_id: Uuid,
    filter: &TaskFilter,
) -> Result<Vec<TaskRow>, sqlx::Error> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"SELECT DISTINCT t.id, t.org_id, t.project_id, t.task_number, t.title, t.description,
                  t.status, t.priority, t.assignee_id, t.reporter_id, t.due_date, t.created_at, t.updated_at
           FROM tasks t"#,
    );
    if filter.tag_id.is_some() {
        qb.push(" JOIN task_tags tt ON tt.task_id = t.id");
    }
    qb.push(" WHERE t.project_id = ");
    qb.push_bind(project_id);

    if let Some(status) = filter.status {
        qb.push(" AND t.status = ");
        qb.push_bind(status);
    }
    if let Some(assignee_id) = filter.assignee_id {
        qb.push(" AND t.assignee_id = ");
        qb.push_bind(assignee_id);
    }
    if let Some(tag_id) = filter.tag_id {
        qb.push(" AND tt.tag_id = ");
        qb.push_bind(tag_id);
    }
    if let Some(search) = &filter.search {
        qb.push(" AND t.title ILIKE ");
        qb.push_bind(format!("%{search}%"));
    }
    qb.push(" ORDER BY t.task_number DESC");

    qb.build_query_as::<TaskRow>().fetch_all(pool).await
}

#[derive(Default)]
pub struct TaskUpdate<'a> {
    pub title: Option<&'a str>,
    pub description: Option<Option<&'a str>>,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub assignee_id: Option<Option<Uuid>>,
    pub due_date: Option<Option<NaiveDate>>,
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    u: TaskUpdate<'_>,
) -> Result<Option<TaskRow>, sqlx::Error> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE tasks SET updated_at = now()");

    if let Some(title) = u.title {
        qb.push(", title = ").push_bind(title);
    }
    if let Some(description) = u.description {
        qb.push(", description = ").push_bind(description);
    }
    if let Some(status) = u.status {
        qb.push(", status = ").push_bind(status);
    }
    if let Some(priority) = u.priority {
        qb.push(", priority = ").push_bind(priority);
    }
    if let Some(assignee_id) = u.assignee_id {
        qb.push(", assignee_id = ").push_bind(assignee_id);
    }
    if let Some(due_date) = u.due_date {
        qb.push(", due_date = ").push_bind(due_date);
    }

    qb.push(" WHERE id = ").push_bind(id);
    qb.push(
        " RETURNING id, org_id, project_id, task_number, title, description, status, priority,
                    assignee_id, reporter_id, due_date, created_at, updated_at",
    );

    qb.build_query_as::<TaskRow>().fetch_optional(pool).await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
