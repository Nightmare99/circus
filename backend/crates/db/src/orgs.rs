use crate::models::{OrgMemberRow, OrgMembershipRow, OrgRow};
use domain::OrgRole;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(pool: &PgPool, name: &str, slug: &str) -> Result<OrgRow, sqlx::Error> {
    sqlx::query_as::<_, OrgRow>(
        r#"INSERT INTO orgs (name, slug) VALUES ($1, $2)
           RETURNING id, name, slug, created_at, updated_at"#,
    )
    .bind(name)
    .bind(slug)
    .fetch_one(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<OrgRow>, sqlx::Error> {
    sqlx::query_as::<_, OrgRow>(
        "SELECT id, name, slug, created_at, updated_at FROM orgs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Option<OrgRow>, sqlx::Error> {
    sqlx::query_as::<_, OrgRow>(
        "SELECT id, name, slug, created_at, updated_at FROM orgs WHERE slug = $1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

pub async fn list_all(pool: &PgPool) -> Result<Vec<OrgRow>, sqlx::Error> {
    sqlx::query_as::<_, OrgRow>(
        "SELECT id, name, slug, created_at, updated_at FROM orgs ORDER BY name",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<OrgRow>, sqlx::Error> {
    sqlx::query_as::<_, OrgRow>(
        r#"SELECT o.id, o.name, o.slug, o.created_at, o.updated_at
           FROM orgs o
           JOIN org_memberships m ON m.org_id = o.id
           WHERE m.user_id = $1
           ORDER BY o.name"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn add_member(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
    role: OrgRole,
) -> Result<OrgMembershipRow, sqlx::Error> {
    sqlx::query_as::<_, OrgMembershipRow>(
        r#"INSERT INTO org_memberships (org_id, user_id, role) VALUES ($1, $2, $3)
           RETURNING id, org_id, user_id, role, created_at"#,
    )
    .bind(org_id)
    .bind(user_id)
    .bind(role)
    .fetch_one(pool)
    .await
}

pub async fn find_membership(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<Option<OrgMembershipRow>, sqlx::Error> {
    sqlx::query_as::<_, OrgMembershipRow>(
        r#"SELECT id, org_id, user_id, role, created_at FROM org_memberships
           WHERE org_id = $1 AND user_id = $2"#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_members(pool: &PgPool, org_id: Uuid) -> Result<Vec<OrgMemberRow>, sqlx::Error> {
    sqlx::query_as::<_, OrgMemberRow>(
        r#"SELECT u.id AS user_id, u.email, u.display_name, m.role
           FROM org_memberships m
           JOIN users u ON u.id = m.user_id
           WHERE m.org_id = $1
           ORDER BY u.display_name"#,
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
}

pub async fn update_member_role(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
    role: OrgRole,
) -> Result<Option<OrgMembershipRow>, sqlx::Error> {
    sqlx::query_as::<_, OrgMembershipRow>(
        r#"UPDATE org_memberships SET role = $3 WHERE org_id = $1 AND user_id = $2
           RETURNING id, org_id, user_id, role, created_at"#,
    )
    .bind(org_id)
    .bind(user_id)
    .bind(role)
    .fetch_optional(pool)
    .await
}

pub async fn remove_member(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM org_memberships WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Count of members with `owner` role, used to prevent removing/demoting the last owner.
pub async fn count_owners(pool: &PgPool, org_id: Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM org_memberships WHERE org_id = $1 AND role = 'owner'")
            .bind(org_id)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}
