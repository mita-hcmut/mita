use std::{net::SocketAddr, sync::Once};

use eyre::WrapErr;
use mita::{config::Config, entrypoint::Server, telemetry};
use wiremock::MockServer;

static TRACING: Once = Once::new();

pub struct TestApp {
    pub http_client: reqwest::Client,
    pub addr: SocketAddr,
    pub moodle_server: MockServer,
    pub id_token: String,
}

impl TestApp {
    pub async fn new() -> eyre::Result<Self> {
        TRACING.call_once(|| {
            telemetry::setup();
        });

        let uuid = uuid::Uuid::new_v4();

        let moodle_server = MockServer::start().await;

        let mut config = Config::test()?;
        config.moodle.url = moodle_server.uri().parse().unwrap();
        config.vault.suffix_path = format!("token-test-{}", uuid);
        let server = Server::build(config.leak()).await?;

        let addr = server.addr();

        tokio::spawn(server);
        let http_client = reqwest::Client::new();

        Ok(Self {
            http_client,
            addr,
            moodle_server,
            id_token: "".into(),
        })
    }

    pub async fn put_token_without_bearer(&self, token: String) -> eyre::Result<reqwest::Response> {
        self.http_client
            .put(format!("http://{}/token", self.addr))
            .form(&[("moodle_token", &token)])
            .send()
            .await
            .wrap_err_with(|| format!("error putting token {token}"))
    }

    pub async fn put_token(&self, token: String) -> eyre::Result<reqwest::Response> {
        self.http_client
            .put(format!("http://{}/token", self.addr))
            .bearer_auth(&self.id_token)
            .form(&[("moodle_token", &token)])
            .send()
            .await
            .wrap_err_with(|| format!("error putting token {token}"))
    }

    pub async fn get_info(&self) -> eyre::Result<reqwest::Response> {
        self.http_client
            .get(format!("http://{}/info", self.addr))
            .bearer_auth(&self.id_token)
            .send()
            .await
            .wrap_err("error getting user info")
    }
}
