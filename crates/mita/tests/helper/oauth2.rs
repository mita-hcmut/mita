use axum::{
    extract::Query,
    routing::{get, post},
    Router, ServiceExt,
};
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::sync::{mpsc, oneshot};

pub async fn get_code() {
    let listener = std::net::TcpListener::bind("localhost:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let redirect_uri = format!("http://localhost:{}", addr.port());

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (code_tx, mut code_rx) = mpsc::channel::<String>(1);

    tokio::spawn(async move {
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

    let client = reqwest::Client::new();
    client
        .post("http://localhost:8080/default/authorize")
        .query(&[
            ("client_id", "client_id"),
            ("response_type", "code"),
            ("redirect_uri", &redirect_uri),
            ("scope", "openid profile email"),
            ("state", "state"),
            ("nonce", "nonce"),
        ])
        .form(&[("username", "khang"), ("claims", "")])
        .send()
        .await
        .unwrap();

    let code = code_rx.recv().await.unwrap();
    drop(code_rx);
    shutdown_tx.send(()).unwrap();

    let res = client
        .post("http://localhost:8080/default/token")
        .form(&[
            ("client_id", "client_id"),
            ("code", &code),
            ("redirect_uri", &redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .unwrap();

    dbg!(res.text().await.unwrap());
}
