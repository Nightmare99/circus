use sqlx::postgres::{PgPool, PgPoolOptions};

pub mod attachments;
pub mod comments;
pub mod invites;
pub mod models;
pub mod orgs;
pub mod projects;
pub mod tags;
pub mod tasks;
pub mod users;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
