use crate::models::{ProjectMemberRow, ProjectMembershipRow, ProjectRow};
use domain::ProjectRole;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    org_id: Uuid,
    key: &str,
    name: &str,
    description: Option<&str>,
) -> Result<ProjectRow, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"INSERT INTO projects (org_id, key, name, description) VALUES ($1, $2, $3, $4)
           RETURNING id, org_id, key, name, description, created_at, updated_at"#,
    )
    .bind(org_id)
    .bind(key)
    .bind(name)
    .bind(description)
    .fetch_one(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        "SELECT id, org_id, key, name, description, created_at, updated_at FROM projects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn list_for_org(pool: &PgPool, org_id: Uuid) -> Result<Vec<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"SELECT id, org_id, key, name, description, created_at, updated_at
           FROM projects WHERE org_id = $1 ORDER BY name"#,
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
}

/// Projects within `org_id` that `user_id` can see: all projects if they're
/// an org owner/admin, otherwise only projects they're explicitly a member of.
pub async fn list_visible_for_user(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
    user_is_org_admin: bool,
) -> Result<Vec<ProjectRow>, sqlx::Error> {
    if user_is_org_admin {
        list_for_org(pool, org_id).await
    } else {
        sqlx::query_as::<_, ProjectRow>(
            r#"SELECT p.id, p.org_id, p.key, p.name, p.description, p.created_at, p.updated_at
               FROM projects p
               JOIN project_memberships pm ON pm.project_id = p.id
               WHERE p.org_id = $1 AND pm.user_id = $2
               ORDER BY p.name"#,
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_all(pool)
        .await
    }
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

pub async fn add_member(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
    role: ProjectRole,
) -> Result<ProjectMembershipRow, sqlx::Error> {
    sqlx::query_as::<_, ProjectMembershipRow>(
        r#"INSERT INTO project_memberships (project_id, user_id, role) VALUES ($1, $2, $3)
           ON CONFLICT (project_id, user_id) DO UPDATE SET role = EXCLUDED.role
           RETURNING id, project_id, user_id, role, created_at"#,
    )
    .bind(project_id)
    .bind(user_id)
    .bind(role)
    .fetch_one(pool)
    .await
}

pub async fn find_membership(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<Option<ProjectMembershipRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectMembershipRow>(
        r#"SELECT id, project_id, user_id, role, created_at FROM project_memberships
           WHERE project_id = $1 AND user_id = $2"#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_members(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ProjectMemberRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectMemberRow>(
        r#"SELECT u.id AS user_id, u.email, u.display_name, pm.role
           FROM project_memberships pm
           JOIN users u ON u.id = pm.user_id
           WHERE pm.project_id = $1
           ORDER BY u.display_name"#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn remove_member(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM project_memberships WHERE project_id = $1 AND user_id = $2")
            .bind(project_id)
            .bind(user_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected())
}

pub async fn next_task_number(pool: &PgPool, project_id: Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r#"UPDATE projects SET next_task_number = next_task_number + 1
           WHERE id = $1 RETURNING next_task_number - 1"#,
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
