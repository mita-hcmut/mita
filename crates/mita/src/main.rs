mod domain;
mod routes;

use axum::{
    routing::{get, post, put},
    Router,
};
use sqlx::sqlite::SqlitePoolOptions;
use std::{net::SocketAddr, time::Duration};

use crate::routes::{root, token::put::register_token};

#[derive(Clone)]
pub struct AppState {
    pub http_client: reqwest::Client,
    pub pool: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        // .connect(option_env!("DATABASE_URL").unwrap())
        .connect("sqlite::memory:")
        .await?;

    let http_client = reqwest::Client::builder().build().unwrap();

    sqlx::migrate!("../../db/migrations").run(&pool).await?;

    let app = Router::new()
        .route("/", get(root))
        .route("/token", put(register_token))
        .with_state(AppState { pool, http_client });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
