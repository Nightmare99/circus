use chrono::{DateTime, Utc};
use domain::{InstanceRole, OrgId, UserId};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub instance_role: InstanceRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserRow {
    pub fn id(&self) -> UserId {
        UserId(self.id)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OrgRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OrgRow {
    pub fn id(&self) -> OrgId {
        OrgId(self.id)
    }
}
