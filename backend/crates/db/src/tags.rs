use crate::models::TagRow;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    project_id: Uuid,
    name: &str,
    color: &str,
) -> Result<TagRow, sqlx::Error> {
    sqlx::query_as::<_, TagRow>(
        r#"INSERT INTO tags (project_id, name, color) VALUES ($1, $2, $3)
           RETURNING id, project_id, name, color"#,
    )
    .bind(project_id)
    .bind(name)
    .bind(color)
    .fetch_one(pool)
    .await
}

pub async fn list_for_project(pool: &PgPool, project_id: Uuid) -> Result<Vec<TagRow>, sqlx::Error> {
    sqlx::query_as::<_, TagRow>(
        "SELECT id, project_id, name, color FROM tags WHERE project_id = $1 ORDER BY name",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn delete(pool: &PgPool, project_id: Uuid, id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tags WHERE id = $1 AND project_id = $2")
        .bind(id)
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

pub async fn list_for_task(pool: &PgPool, task_id: Uuid) -> Result<Vec<TagRow>, sqlx::Error> {
    sqlx::query_as::<_, TagRow>(
        r#"SELECT t.id, t.project_id, t.name, t.color
           FROM tags t
           JOIN task_tags tt ON tt.tag_id = t.id
           WHERE tt.task_id = $1
           ORDER BY t.name"#,
    )
    .bind(task_id)
    .fetch_all(pool)
    .await
}

pub async fn set_task_tags(
    pool: &PgPool,
    task_id: Uuid,
    tag_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM task_tags WHERE task_id = $1")
        .bind(task_id)
        .execute(&mut *tx)
        .await?;
    for tag_id in tag_ids {
        sqlx::query("INSERT INTO task_tags (task_id, tag_id) VALUES ($1, $2)")
            .bind(task_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await
}
