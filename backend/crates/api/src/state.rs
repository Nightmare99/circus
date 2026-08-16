use crate::storage::Storage;
use sqlx::PgPool;
use std::sync::Arc;

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
}

impl std::ops::Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
