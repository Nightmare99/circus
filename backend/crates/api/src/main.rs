mod admin;
mod auth;
mod config;
mod error;
mod events;
mod orgs;
mod projects;
mod routes;
mod serde_util;
mod state;
mod storage;
mod tasks;
mod util;
mod ws;

use config::Config;
use domain::InstanceRole;
use state::{AppState, AppStateInner};
use std::sync::Arc;
use storage::Storage;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env();

    let pool = db::connect(&config.database_url).await?;
    db::migrate(&pool).await?;
    tracing::info!("database migrated");

    let storage = Storage::new(&config.storage_dir).await?;

    bootstrap_admin(&pool, &config).await?;

    let (events_tx, _) = tokio::sync::broadcast::channel(1024);

    let state = AppState(Arc::new(AppStateInner {
        pool,
        jwt_secret: config.jwt_secret.clone(),
        access_token_ttl_minutes: config.access_token_ttl_minutes,
        refresh_token_ttl_days: config.refresh_token_ttl_days,
        storage,
        max_upload_mb: config.max_upload_mb,
        cookie_secure: config.cookie_secure,
        events: events_tx,
    }));

    let app = routes::router(state, config.static_dir.clone())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "circus-api listening");
    axum::serve(listener, app).await?;

    Ok(())
}

/// Creates the instance's first superadmin from `BOOTSTRAP_ADMIN_EMAIL` /
/// `BOOTSTRAP_ADMIN_PASSWORD` if no superadmin exists yet. Safe to leave
/// those env vars set permanently — this is a no-op once a superadmin
/// already exists, which is how the Helm chart wires initial-admin creds.
async fn bootstrap_admin(pool: &sqlx::PgPool, config: &Config) -> anyhow::Result<()> {
    let (Some(email), Some(password)) = (
        config.bootstrap_admin_email.as_deref(),
        config.bootstrap_admin_password.as_deref(),
    ) else {
        return Ok(());
    };
    if db::users::any_superadmin_exists(pool).await? {
        return Ok(());
    }
    let email = email.trim().to_lowercase();
    if password.len() < 8 {
        anyhow::bail!("BOOTSTRAP_ADMIN_PASSWORD must be at least 8 characters");
    }
    let hash = auth::password::hash(password).map_err(|e| anyhow::anyhow!(e))?;
    if let Some(existing) = db::users::find_by_email(pool, &email).await? {
        db::users::update_instance_role(pool, existing.id, InstanceRole::Superadmin).await?;
        tracing::info!(%email, "promoted existing user to superadmin");
    } else {
        db::users::create(
            pool,
            &email,
            &hash,
            &config.bootstrap_admin_name,
            InstanceRole::Superadmin,
        )
        .await?;
        tracing::info!(%email, "created bootstrap superadmin");
    }
    Ok(())
}
