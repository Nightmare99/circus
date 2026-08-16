use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde_json::{json, Value};
use sqlx::PgPool;

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(pool)
}

async fn healthz() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn readyz(State(pool): State<PgPool>) -> (StatusCode, Json<Value>) {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "ready" }))),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "not_ready" })),
        ),
    }
}
