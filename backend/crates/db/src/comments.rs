use crate::models::CommentRow;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    task_id: Uuid,
    author_id: Uuid,
    body: &str,
) -> Result<CommentRow, sqlx::Error> {
    sqlx::query_as::<_, CommentRow>(
        r#"INSERT INTO comments (task_id, author_id, body) VALUES ($1, $2, $3)
           RETURNING id, task_id, author_id, body, created_at, updated_at"#,
    )
    .bind(task_id)
    .bind(author_id)
    .bind(body)
    .fetch_one(pool)
    .await
}

pub async fn list_for_task(pool: &PgPool, task_id: Uuid) -> Result<Vec<CommentRow>, sqlx::Error> {
    sqlx::query_as::<_, CommentRow>(
        r#"SELECT id, task_id, author_id, body, created_at, updated_at
           FROM comments WHERE task_id = $1 ORDER BY created_at ASC"#,
    )
    .bind(task_id)
    .fetch_all(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<CommentRow>, sqlx::Error> {
    sqlx::query_as::<_, CommentRow>(
        "SELECT id, task_id, author_id, body, created_at, updated_at FROM comments WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn update_body(
    pool: &PgPool,
    id: Uuid,
    body: &str,
) -> Result<Option<CommentRow>, sqlx::Error> {
    sqlx::query_as::<_, CommentRow>(
        r#"UPDATE comments SET body = $2, updated_at = now() WHERE id = $1
           RETURNING id, task_id, author_id, body, created_at, updated_at"#,
    )
    .bind(id)
    .bind(body)
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM comments WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
