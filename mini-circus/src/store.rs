use crate::models::{Board, Task};
use chrono::Utc;
use common::{Priority, TaskStatus};
use sqlx::SqlitePool;

const TASK_COLUMNS: &str =
    "id, board_id, title, description, status, priority, assignee, created_at, updated_at";

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("board {0:?} not found")]
    BoardNotFound(String),
    #[error("task {0} not found")]
    TaskNotFound(i64),
    #[error("a board named {0:?} already exists")]
    BoardNameTaken(String),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

// ---- boards ----------------------------------------------------------

pub async fn create_board(
    pool: &SqlitePool,
    name: &str,
    description: Option<&str>,
) -> Result<Board, StoreError> {
    let now = Utc::now();
    sqlx::query_as::<_, Board>(
        "INSERT INTO boards (name, description, created_at, updated_at) VALUES (?, ?, ?, ?)
         RETURNING id, name, description, created_at, updated_at",
    )
    .bind(name)
    .bind(description)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            StoreError::BoardNameTaken(name.to_string())
        }
        _ => StoreError::Db(e),
    })
}

pub async fn list_boards(pool: &SqlitePool) -> Result<Vec<Board>, StoreError> {
    Ok(sqlx::query_as::<_, Board>(
        "SELECT id, name, description, created_at, updated_at FROM boards ORDER BY name",
    )
    .fetch_all(pool)
    .await?)
}

/// Boards can be referenced by numeric id or by their (unique) name -
/// whichever is more convenient on the command line.
pub async fn resolve_board(pool: &SqlitePool, reference: &str) -> Result<Board, StoreError> {
    let found = if let Ok(id) = reference.parse::<i64>() {
        sqlx::query_as::<_, Board>(
            "SELECT id, name, description, created_at, updated_at FROM boards WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
    } else {
        None
    };
    let found =
        match found {
            Some(b) => Some(b),
            None => sqlx::query_as::<_, Board>(
                "SELECT id, name, description, created_at, updated_at FROM boards WHERE name = ?",
            )
            .bind(reference)
            .fetch_optional(pool)
            .await?,
        };
    found.ok_or_else(|| StoreError::BoardNotFound(reference.to_string()))
}

pub async fn delete_board(pool: &SqlitePool, id: i64) -> Result<(), StoreError> {
    let result = sqlx::query("DELETE FROM boards WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(StoreError::BoardNotFound(id.to_string()));
    }
    Ok(())
}

// ---- tasks -------------------------------------------------------------

pub struct NewTask<'a> {
    pub board_id: i64,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub priority: Priority,
    pub assignee: Option<&'a str>,
}

pub async fn create_task(pool: &SqlitePool, t: NewTask<'_>) -> Result<Task, StoreError> {
    let now = Utc::now();
    let query = format!(
        "INSERT INTO tasks (board_id, title, description, status, priority, assignee, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         RETURNING {TASK_COLUMNS}"
    );
    Ok(sqlx::query_as::<_, Task>(&query)
        .bind(t.board_id)
        .bind(t.title)
        .bind(t.description)
        .bind(TaskStatus::Pending)
        .bind(t.priority)
        .bind(t.assignee)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?)
}

pub async fn get_task(pool: &SqlitePool, id: i64) -> Result<Task, StoreError> {
    let query = format!("SELECT {TASK_COLUMNS} FROM tasks WHERE id = ?");
    sqlx::query_as::<_, Task>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(StoreError::TaskNotFound(id))
}

#[derive(Default)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub assignee: Option<String>,
}

pub async fn list_tasks(
    pool: &SqlitePool,
    board_id: i64,
    filter: &TaskFilter,
) -> Result<Vec<Task>, StoreError> {
    let mut query = format!("SELECT {TASK_COLUMNS} FROM tasks WHERE board_id = ?");
    if filter.status.is_some() {
        query.push_str(" AND status = ?");
    }
    if filter.assignee.is_some() {
        query.push_str(" AND assignee = ?");
    }
    query.push_str(" ORDER BY id");

    let mut q = sqlx::query_as::<_, Task>(&query).bind(board_id);
    if let Some(status) = filter.status {
        q = q.bind(status);
    }
    if let Some(assignee) = &filter.assignee {
        q = q.bind(assignee);
    }
    Ok(q.fetch_all(pool).await?)
}

#[derive(Default)]
pub struct TaskPatch<'a> {
    pub title: Option<&'a str>,
    pub description: Option<Option<&'a str>>,
    pub priority: Option<Priority>,
    pub status: Option<TaskStatus>,
    pub assignee: Option<Option<&'a str>>,
}

pub async fn update_task(
    pool: &SqlitePool,
    id: i64,
    patch: TaskPatch<'_>,
) -> Result<Task, StoreError> {
    let mut set_clauses = vec!["updated_at = ?".to_string()];
    if patch.title.is_some() {
        set_clauses.push("title = ?".to_string());
    }
    if patch.description.is_some() {
        set_clauses.push("description = ?".to_string());
    }
    if patch.priority.is_some() {
        set_clauses.push("priority = ?".to_string());
    }
    if patch.status.is_some() {
        set_clauses.push("status = ?".to_string());
    }
    if patch.assignee.is_some() {
        set_clauses.push("assignee = ?".to_string());
    }

    let query = format!(
        "UPDATE tasks SET {} WHERE id = ? RETURNING {TASK_COLUMNS}",
        set_clauses.join(", ")
    );
    let mut q = sqlx::query_as::<_, Task>(&query).bind(Utc::now());
    if let Some(title) = patch.title {
        q = q.bind(title);
    }
    if let Some(description) = patch.description {
        q = q.bind(description);
    }
    if let Some(priority) = patch.priority {
        q = q.bind(priority);
    }
    if let Some(status) = patch.status {
        q = q.bind(status);
    }
    if let Some(assignee) = patch.assignee {
        q = q.bind(assignee);
    }
    q = q.bind(id);

    q.fetch_optional(pool)
        .await?
        .ok_or(StoreError::TaskNotFound(id))
}

/// Atomically claims the oldest unassigned, pending task on a board -
/// assigns it and marks it in progress in a single statement, so it's safe
/// to call concurrently from multiple processes without double-assignment.
pub async fn claim_next_task(
    pool: &SqlitePool,
    board_id: i64,
    assignee: &str,
) -> Result<Option<Task>, StoreError> {
    let query = format!(
        "UPDATE tasks SET assignee = ?, status = ?, updated_at = ?
         WHERE id = (
             SELECT id FROM tasks
             WHERE board_id = ? AND assignee IS NULL AND status = ?
             ORDER BY id ASC LIMIT 1
         )
         RETURNING {TASK_COLUMNS}"
    );
    Ok(sqlx::query_as::<_, Task>(&query)
        .bind(assignee)
        .bind(TaskStatus::InProgress)
        .bind(Utc::now())
        .bind(board_id)
        .bind(TaskStatus::Pending)
        .fetch_optional(pool)
        .await?)
}

pub async fn delete_task(pool: &SqlitePool, id: i64) -> Result<(), StoreError> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(StoreError::TaskNotFound(id));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    async fn test_pool() -> SqlitePool {
        let path = std::env::temp_dir().join(format!("mini-circus-test-{}.db", uuid_like()));
        db::connect(&path).await.expect("connect")
    }

    fn uuid_like() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{nanos}-{:?}", std::thread::current().id())
    }

    #[tokio::test]
    async fn create_and_resolve_board_by_id_and_name() {
        let pool = test_pool().await;
        let board = create_board(&pool, "backlog", Some("desc")).await.unwrap();

        let by_id = resolve_board(&pool, &board.id.to_string()).await.unwrap();
        assert_eq!(by_id.id, board.id);

        let by_name = resolve_board(&pool, "backlog").await.unwrap();
        assert_eq!(by_name.id, board.id);

        assert!(matches!(
            resolve_board(&pool, "missing").await,
            Err(StoreError::BoardNotFound(_))
        ));
    }

    #[tokio::test]
    async fn duplicate_board_name_is_rejected() {
        let pool = test_pool().await;
        create_board(&pool, "dup", None).await.unwrap();
        assert!(matches!(
            create_board(&pool, "dup", None).await,
            Err(StoreError::BoardNameTaken(_))
        ));
    }

    #[tokio::test]
    async fn new_task_defaults_to_pending_medium() {
        let pool = test_pool().await;
        let board = create_board(&pool, "b", None).await.unwrap();
        let task = create_task(
            &pool,
            NewTask {
                board_id: board.id,
                title: "do the thing",
                description: None,
                priority: Priority::Medium,
                assignee: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, Priority::Medium);
        assert_eq!(task.assignee, None);
    }

    #[tokio::test]
    async fn update_task_only_touches_provided_fields() {
        let pool = test_pool().await;
        let board = create_board(&pool, "b", None).await.unwrap();
        let task = create_task(
            &pool,
            NewTask {
                board_id: board.id,
                title: "title",
                description: Some("desc"),
                priority: Priority::Low,
                assignee: Some("alice"),
            },
        )
        .await
        .unwrap();

        let updated = update_task(
            &pool,
            task.id,
            TaskPatch {
                priority: Some(Priority::Urgent),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(updated.priority, Priority::Urgent);
        // untouched fields survive the partial update
        assert_eq!(updated.title, "title");
        assert_eq!(updated.description.as_deref(), Some("desc"));
        assert_eq!(updated.assignee.as_deref(), Some("alice"));
        assert_eq!(updated.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn update_task_can_clear_a_nullable_field() {
        let pool = test_pool().await;
        let board = create_board(&pool, "b", None).await.unwrap();
        let task = create_task(
            &pool,
            NewTask {
                board_id: board.id,
                title: "title",
                description: None,
                priority: Priority::Medium,
                assignee: Some("alice"),
            },
        )
        .await
        .unwrap();

        let cleared = update_task(
            &pool,
            task.id,
            TaskPatch {
                assignee: Some(None), // explicit clear, distinct from "don't touch"
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(cleared.assignee, None);
    }

    #[tokio::test]
    async fn claim_next_task_is_atomic_and_exhausts_cleanly() {
        let pool = test_pool().await;
        let board = create_board(&pool, "b", None).await.unwrap();
        for i in 0..3 {
            create_task(
                &pool,
                NewTask {
                    board_id: board.id,
                    title: &format!("task {i}"),
                    description: None,
                    priority: Priority::Medium,
                    assignee: None,
                },
            )
            .await
            .unwrap();
        }

        let first = claim_next_task(&pool, board.id, "worker-a")
            .await
            .unwrap()
            .expect("a task");
        let second = claim_next_task(&pool, board.id, "worker-b")
            .await
            .unwrap()
            .expect("a task");
        let third = claim_next_task(&pool, board.id, "worker-c")
            .await
            .unwrap()
            .expect("a task");

        // three distinct tasks, each now in progress and assigned to whoever claimed it
        let ids: std::collections::HashSet<_> =
            [first.id, second.id, third.id].into_iter().collect();
        assert_eq!(ids.len(), 3);
        assert_eq!(first.status, TaskStatus::InProgress);
        assert_eq!(first.assignee.as_deref(), Some("worker-a"));

        // nothing left to claim
        let none_left = claim_next_task(&pool, board.id, "worker-d").await.unwrap();
        assert!(none_left.is_none());
    }

    #[tokio::test]
    async fn claim_skips_already_assigned_tasks() {
        let pool = test_pool().await;
        let board = create_board(&pool, "b", None).await.unwrap();
        create_task(
            &pool,
            NewTask {
                board_id: board.id,
                title: "already taken",
                description: None,
                priority: Priority::Medium,
                assignee: Some("someone"),
            },
        )
        .await
        .unwrap();

        let claimed = claim_next_task(&pool, board.id, "worker").await.unwrap();
        assert!(claimed.is_none(), "pre-assigned task must not be claimable");
    }

    #[tokio::test]
    async fn deleting_a_board_cascades_to_its_tasks() {
        let pool = test_pool().await;
        let board = create_board(&pool, "b", None).await.unwrap();
        let task = create_task(
            &pool,
            NewTask {
                board_id: board.id,
                title: "t",
                description: None,
                priority: Priority::Medium,
                assignee: None,
            },
        )
        .await
        .unwrap();

        delete_board(&pool, board.id).await.unwrap();

        assert!(matches!(
            get_task(&pool, task.id).await,
            Err(StoreError::TaskNotFound(_))
        ));
    }
}
