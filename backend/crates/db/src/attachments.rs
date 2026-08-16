use crate::models::AttachmentRow;
use sqlx::PgPool;
use uuid::Uuid;

pub struct NewAttachment<'a> {
    pub id: Uuid,
    pub task_id: Uuid,
    pub uploaded_by: Uuid,
    pub file_name: &'a str,
    pub content_type: &'a str,
    pub size_bytes: i64,
    pub storage_key: &'a str,
}

pub async fn create(pool: &PgPool, a: NewAttachment<'_>) -> Result<AttachmentRow, sqlx::Error> {
    sqlx::query_as::<_, AttachmentRow>(
        r#"INSERT INTO attachments (id, task_id, uploaded_by, file_name, content_type, size_bytes, storage_key)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, task_id, uploaded_by, file_name, content_type, size_bytes, storage_key, created_at"#,
    )
    .bind(a.id)
    .bind(a.task_id)
    .bind(a.uploaded_by)
    .bind(a.file_name)
    .bind(a.content_type)
    .bind(a.size_bytes)
    .bind(a.storage_key)
    .fetch_one(pool)
    .await
}

pub async fn list_for_task(
    pool: &PgPool,
    task_id: Uuid,
) -> Result<Vec<AttachmentRow>, sqlx::Error> {
    sqlx::query_as::<_, AttachmentRow>(
        r#"SELECT id, task_id, uploaded_by, file_name, content_type, size_bytes, storage_key, created_at
           FROM attachments WHERE task_id = $1 ORDER BY created_at ASC"#,
    )
    .bind(task_id)
    .fetch_all(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AttachmentRow>, sqlx::Error> {
    sqlx::query_as::<_, AttachmentRow>(
        r#"SELECT id, task_id, uploaded_by, file_name, content_type, size_bytes, storage_key, created_at
           FROM attachments WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM attachments WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
