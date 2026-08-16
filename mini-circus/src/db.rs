use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// `$MINI_CIRCUS_DB`, else `./.mini-circus/board.db` — a board scoped to the
/// current directory by default, so it just works when run from a repo
/// without any setup step.
pub fn default_db_path() -> PathBuf {
    if let Ok(path) = std::env::var("MINI_CIRCUS_DB") {
        return PathBuf::from(path);
    }
    PathBuf::from(".mini-circus").join("board.db")
}

pub async fn connect(path: &Path) -> anyhow::Result<SqlitePool> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        // Multiple processes (e.g. several workers polling the same board)
        // may write concurrently; wait briefly instead of failing outright
        // on "database is locked".
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
