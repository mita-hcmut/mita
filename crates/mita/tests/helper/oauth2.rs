use std::future::Future;

use axum::{extract::Query, routing::get, Router};
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::sync::{mpsc, oneshot};

#[derive(Deserialize)]
pub struct TokenResult {
    pub token_type: String,
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

pub async fn get_code(username: &str, claims: &str) -> TokenResult {
    const OAUTH_ADDR: &str = "http://localhost:8443/default";

    let (redirect_uri, code) = oneshot_redirect_server();
    let client = reqwest::Client::new();
    client
        .post(&format!("{}/authorize", OAUTH_ADDR))
        .query(&[
            ("client_id", "client_id"),
            ("response_type", "code"),
            ("redirect_uri", &redirect_uri),
            ("scope", "openid"),
            ("state", "state"),
            ("nonce", "nonce"),
        ])
        .form(&[("username", username), ("claims", claims)])
        .send()
        .await
        .unwrap();

    let code = code.await;

    let res = client
        .post(&format!("{}/token", OAUTH_ADDR))
        .form(&[
            ("client_id", "client_id"),
            ("code", &code),
            ("redirect_uri", &redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .unwrap();

    res.json().await.unwrap()
}

fn oneshot_redirect_server() -> (String, impl Future<Output = String>) {
    let listener = std::net::TcpListener::bind("localhost:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let redirect_uri = format!("http://localhost:{}", addr.port());

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (code_tx, mut code_rx) = mpsc::channel::<String>(1);

    let handle = tokio::spawn(async move {
        #[derive(Deserialize)]
        struct OAuthQuery {
            code: String,
        }

        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(
                Router::new()
                    .route(
                        "/",
                        get(|Query(q): Query<OAuthQuery>| async move {
                            code_tx.send(q.code).await.unwrap();
                            StatusCode::OK
                        }),
                    )
                    .into_make_service(),
            )
            .with_graceful_shutdown(async {
                shutdown_rx.await.unwrap();
            })
            .await
            .unwrap();
    });

    let code = async move {
        let code = code_rx.recv().await.unwrap();
        drop(code_rx);
        shutdown_tx.send(()).unwrap();
        handle.await.unwrap();
        code
    };

    (redirect_uri, code)
}
