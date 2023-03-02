use std::{net::SocketAddr, time::Duration};

use sqlx::sqlite::SqlitePoolOptions;

use crate::{app_state::AppState, routes::router::app_router};

pub async fn build() -> eyre::Result<()> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        // .connect(option_env!("DATABASE_URL").unwrap())
        .connect("sqlite::memory:")
        .await?;

    sqlx::migrate!("../../db/migrations").run(&pool).await?;

    let http_client = reqwest::Client::builder().build().unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::debug!("listening on {}", addr);

    let app = app_router().with_state(AppState { http_client, pool });

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
