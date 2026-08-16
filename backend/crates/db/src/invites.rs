use crate::models::InviteRow;
use chrono::{DateTime, Utc};
use domain::OrgRole;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    org_id: Uuid,
    email: &str,
    role: OrgRole,
    token: &str,
    invited_by: Uuid,
    expires_at: DateTime<Utc>,
) -> Result<InviteRow, sqlx::Error> {
    sqlx::query_as::<_, InviteRow>(
        r#"INSERT INTO invites (org_id, email, role, token, invited_by, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, org_id, email, role, token, invited_by, expires_at, accepted_at, created_at"#,
    )
    .bind(org_id)
    .bind(email)
    .bind(role)
    .bind(token)
    .bind(invited_by)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

pub async fn find_by_token(pool: &PgPool, token: &str) -> Result<Option<InviteRow>, sqlx::Error> {
    sqlx::query_as::<_, InviteRow>(
        r#"SELECT id, org_id, email, role, token, invited_by, expires_at, accepted_at, created_at
           FROM invites WHERE token = $1"#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await
}

pub async fn list_pending(pool: &PgPool, org_id: Uuid) -> Result<Vec<InviteRow>, sqlx::Error> {
    sqlx::query_as::<_, InviteRow>(
        r#"SELECT id, org_id, email, role, token, invited_by, expires_at, accepted_at, created_at
           FROM invites WHERE org_id = $1 AND accepted_at IS NULL ORDER BY created_at DESC"#,
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
}

pub async fn mark_accepted(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE invites SET accepted_at = now() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn revoke(pool: &PgPool, org_id: Uuid, id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM invites WHERE id = $1 AND org_id = $2")
        .bind(id)
        .bind(org_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
