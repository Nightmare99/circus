use crate::{admin, auth, orgs, projects, state::AppState, tasks};
use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::{get, patch, post, put},
    Json, Router,
};
use serde_json::{json, Value};
use tower_http::services::{ServeDir, ServeFile};

pub fn router(state: AppState, static_dir: Option<String>) -> Router {
    let max_upload_bytes = state.max_upload_mb * 1024 * 1024 + 1024 * 1024;

    let api = Router::new()
        .route("/auth/register", post(auth::handlers::register))
        .route("/auth/login", post(auth::handlers::login))
        .route("/auth/refresh", post(auth::handlers::refresh))
        .route("/auth/logout", post(auth::handlers::logout))
        .route("/auth/me", get(auth::handlers::me))
        .route("/invites/accept", post(orgs::handlers::accept_invite))
        .route(
            "/orgs",
            post(orgs::handlers::create_org).get(orgs::handlers::list_orgs),
        )
        .route("/orgs/{org_id}", get(orgs::handlers::get_org))
        .route("/orgs/{org_id}/members", get(orgs::handlers::list_members))
        .route(
            "/orgs/{org_id}/members/{user_id}",
            patch(orgs::handlers::update_member_role).delete(orgs::handlers::remove_member),
        )
        .route(
            "/orgs/{org_id}/invites",
            post(orgs::handlers::create_invite).get(orgs::handlers::list_invites),
        )
        .route(
            "/orgs/{org_id}/invites/{invite_id}",
            axum::routing::delete(orgs::handlers::revoke_invite),
        )
        .route(
            "/orgs/{org_id}/projects",
            post(projects::handlers::create_project).get(projects::handlers::list_projects),
        )
        .route(
            "/projects/{project_id}",
            get(projects::handlers::get_project).delete(projects::handlers::delete_project),
        )
        .route(
            "/projects/{project_id}/members",
            get(projects::handlers::list_members).post(projects::handlers::add_member),
        )
        .route(
            "/projects/{project_id}/members/{user_id}",
            axum::routing::delete(projects::handlers::remove_member),
        )
        .route(
            "/projects/{project_id}/tags",
            get(projects::handlers::list_tags).post(projects::handlers::create_tag),
        )
        .route(
            "/projects/{project_id}/tags/{tag_id}",
            axum::routing::delete(projects::handlers::delete_tag),
        )
        .route(
            "/projects/{project_id}/tasks",
            get(tasks::handlers::list_tasks).post(tasks::handlers::create_task),
        )
        .route(
            "/tasks/{task_id}",
            get(tasks::handlers::get_task)
                .patch(tasks::handlers::update_task)
                .delete(tasks::handlers::delete_task),
        )
        .route("/tasks/{task_id}/tags", put(tasks::handlers::set_task_tags))
        .route(
            "/tasks/{task_id}/comments",
            get(tasks::comments::list_comments).post(tasks::comments::create_comment),
        )
        .route(
            "/tasks/{task_id}/comments/{comment_id}",
            patch(tasks::comments::update_comment).delete(tasks::comments::delete_comment),
        )
        .route(
            "/tasks/{task_id}/attachments",
            get(tasks::attachments::list_attachments).post(tasks::attachments::upload_attachment),
        )
        .route(
            "/tasks/{task_id}/attachments/{attachment_id}",
            get(tasks::attachments::download_attachment)
                .delete(tasks::attachments::delete_attachment),
        )
        .route("/admin/orgs", get(admin::handlers::list_orgs))
        .route("/admin/users", get(admin::handlers::list_users))
        .route(
            "/admin/users/{user_id}",
            patch(admin::handlers::update_user_role),
        )
        .layer(DefaultBodyLimit::max(max_upload_bytes));

    let mut app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .nest("/api", api)
        .with_state(state);

    if let Some(dir) = static_dir {
        // `.fallback()` (not `.not_found_service()`, which forces 404) so client-side
        // routes like /orgs/{id} serve index.html with a normal 200.
        let index = format!("{dir}/index.html");
        let serve_dir = ServeDir::new(dir).fallback(ServeFile::new(index));
        app = app.fallback_service(serve_dir);
    }

    app
}

async fn healthz() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "ready" }))),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "not_ready" })),
        ),
    }
}
