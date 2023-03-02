#[derive(Clone)]
pub struct AppState {
    pub http_client: reqwest::Client,
    pub pool: sqlx::SqlitePool,
}
