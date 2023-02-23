mod domain;
mod routes;

use axum::{
    routing::{get, post, put},
    Router,
};
use sqlx::sqlite::SqlitePoolOptions;
use std::{net::SocketAddr, time::Duration};

use crate::routes::{get_tokens::get_tokens, put_token::put_token, root};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(env!("DATABASE_URL"))
        .await?;

    sqlx::migrate!("../../db/migrations").run(&pool).await?;

    let app = Router::new()
        .route("/", get(root))
        .route("/token", put(put_token))
        .route("/tokens", get(get_tokens))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
