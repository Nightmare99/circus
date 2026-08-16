use crate::models::UserRow;
use domain::InstanceRole;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    display_name: &str,
    instance_role: InstanceRole,
) -> Result<UserRow, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"INSERT INTO users (email, password_hash, display_name, instance_role)
           VALUES ($1, $2, $3, $4)
           RETURNING id, email, password_hash, display_name, instance_role, created_at, updated_at"#,
    )
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .bind(instance_role)
    .fetch_one(pool)
    .await
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"SELECT id, email, password_hash, display_name, instance_role, created_at, updated_at
           FROM users WHERE email = $1"#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"SELECT id, email, password_hash, display_name, instance_role, created_at, updated_at
           FROM users WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn any_superadmin_exists(pool: &PgPool) -> Result<bool, sqlx::Error> {
    let row: (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE instance_role = 'superadmin')")
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

pub async fn list_all(pool: &PgPool) -> Result<Vec<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"SELECT id, email, password_hash, display_name, instance_role, created_at, updated_at
           FROM users ORDER BY created_at"#,
    )
    .fetch_all(pool)
    .await
}

pub async fn update_instance_role(
    pool: &PgPool,
    id: Uuid,
    instance_role: InstanceRole,
) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"UPDATE users SET instance_role = $2, updated_at = now() WHERE id = $1
           RETURNING id, email, password_hash, display_name, instance_role, created_at, updated_at"#,
    )
    .bind(id)
    .bind(instance_role)
    .fetch_optional(pool)
    .await
}
