use crate::events::ProjectEvent;
use crate::storage::Storage;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState(pub Arc<AppStateInner>);

pub struct AppStateInner {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub access_token_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
    pub storage: Storage,
    pub max_upload_mb: usize,
    pub cookie_secure: bool,
    pub events: broadcast::Sender<ProjectEvent>,
}

impl AppStateInner {
    /// Best-effort notify: errors only when there are no active WebSocket
    /// subscribers, which is the common case and not worth logging.
    pub fn notify_project(&self, project_id: uuid::Uuid) {
        let _ = self.events.send(ProjectEvent { project_id });
    }
}

impl std::ops::Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
