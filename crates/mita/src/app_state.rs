use std::sync::Arc;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub http_client: reqwest::Client,
    pub pool: sqlx::SqlitePool,
    pub config: Arc<Config>,
}
