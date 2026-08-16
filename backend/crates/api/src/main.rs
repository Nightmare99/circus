mod config;
mod routes;

use config::Config;
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

    let app = routes::router(pool)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "circus-api listening");
    axum::serve(listener, app).await?;

    Ok(())
}
