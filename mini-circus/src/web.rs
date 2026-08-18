//! Optional, read-only live dashboard (`mini-circus serve`). Every mutation
//! to the board happens in a separate, short-lived `mini-circus` CLI
//! process, so unlike circus's API server there's no single long-running
//! process that can broadcast "a task changed" from inside the mutation
//! itself. Instead this watches the SQLite file's directory for writes (via
//! `notify`, whichever OS-native mechanism is available) and pushes a ping
//! over a WebSocket to connected browsers, who then refetch over the plain
//! JSON API below - push-triggered, pull-verified.

use crate::models::{Board, Task, TaskDetail};
use crate::store::{self, StoreError, TaskFilter};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use notify::{RecursiveMode, Watcher};
use sqlx::SqlitePool;
use std::path::Path as FsPath;
use std::sync::Arc;
use tokio::sync::broadcast;

const DASHBOARD_HTML: &str = include_str!("web/dashboard.html");

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
    changes: broadcast::Sender<()>,
}

struct ApiError(StoreError);

impl From<StoreError> for ApiError {
    fn from(e: StoreError) -> Self {
        ApiError(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            StoreError::BoardNotFound(_)
            | StoreError::TaskNotFound(_)
            | StoreError::CommentNotFound(_) => StatusCode::NOT_FOUND,
            StoreError::BoardNameTaken(_) => StatusCode::CONFLICT,
            StoreError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (
            status,
            Json(serde_json::json!({ "error": self.0.to_string() })),
        )
            .into_response()
    }
}

pub async fn run(
    pool: SqlitePool,
    db_path: std::path::PathBuf,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    let (changes_tx, _) = broadcast::channel(64);

    // Keep the watcher alive for the process lifetime by holding it in this
    // function's scope across the `axum::serve` await below.
    let watch_dir = db_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| FsPath::new("."))
        .to_path_buf();
    std::fs::create_dir_all(&watch_dir)?;

    let watcher_tx = changes_tx.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if res.is_ok() {
            // Errors here just mean no receivers are currently connected.
            let _ = watcher_tx.send(());
        }
    })?;
    watcher.watch(&watch_dir, RecursiveMode::NonRecursive)?;

    let state = Arc::new(AppState {
        pool,
        changes: changes_tx,
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/api/boards", get(list_boards))
        .route("/api/boards/{board}/tasks", get(list_tasks))
        .route("/api/tasks/{id}", get(get_task))
        .route("/ws", get(ws_upgrade))
        .with_state(state);

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("mini-circus dashboard: http://{addr}  (Ctrl+C to stop)");
    axum::serve(listener, app).await?;

    // Keep `watcher` alive until here; dropping it earlier would stop events.
    drop(watcher);
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn list_boards(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Board>>, ApiError> {
    Ok(Json(store::list_boards(&state.pool).await?))
}

async fn list_tasks(
    State(state): State<Arc<AppState>>,
    Path(board): Path<String>,
) -> Result<Json<Vec<Task>>, ApiError> {
    let board = store::resolve_board(&state.pool, &board).await?;
    let tasks = store::list_tasks(&state.pool, board.id, &TaskFilter::default()).await?;
    Ok(Json(tasks))
}

async fn get_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<TaskDetail>, ApiError> {
    let task = store::get_task(&state.pool, id).await?;
    let comments = store::list_comments(&state.pool, id).await?;
    Ok(Json(TaskDetail { task, comments }))
}

async fn ws_upgrade(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.changes.subscribe();
    loop {
        tokio::select! {
            change = rx.recv() => {
                match change {
                    Ok(()) => {
                        if socket.send(Message::Text("changed".into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    None | Some(Err(_)) => break,
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(_)) => {}
                }
            }
        }
    }
}
