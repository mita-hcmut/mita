use std::{future::Future, net::SocketAddr, sync::Arc, time::Duration};

use eyre::WrapErr;
use futures::future::BoxFuture;
use sqlx::sqlite::SqlitePoolOptions;

use crate::{app_state::AppState, config::Config, routes::router::app_router};

pub struct Server {
    addr: SocketAddr,
    axum_server: BoxFuture<'static, eyre::Result<()>>,
}

impl Server {
    pub async fn build(config: &'static Config) -> eyre::Result<Self> {
        let addr = format!("{}:{}", &config.app.hostname, config.app.port).parse()?;

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            // .connect(option_env!("DATABASE_URL").unwrap())
            .connect(&config.database.url)
            .await?;

        sqlx::migrate!("../../db/migrations").run(&pool).await?;

        let http_client = reqwest::Client::builder().build().unwrap();

        let app = app_router().with_state(AppState {
            http_client,
            pool,
            config,
        });

        let server = axum::Server::bind(&addr).serve(app.into_make_service());

        tracing::info!("listening on {}", server.local_addr());

        Ok(Self {
            addr: server.local_addr(),
            axum_server: Box::pin(async { server.await.wrap_err("error running server") }),
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Future for Server {
    type Output = eyre::Result<()>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.axum_server.as_mut().poll(cx)
    }
}
