use axum::extract::State;
use axum_auth::AuthBearer;
use reqwest::StatusCode;
use serde::Serialize;

use crate::app_state::AppState;

pub async fn register_token(AuthBearer(token): AuthBearer, state: State<AppState>) -> StatusCode {
    #[derive(Serialize)]
    struct LoginRequest<'a> {
        role: &'a str,
        jwt: &'a str,
    }

    let res = state
        .0
        .http_client
        .post("http://localhost:8200/v1/auth/jwt/login")
        .json(&LoginRequest {
            role: "user",
            jwt: &token,
        })
        .send()
        .await
        .unwrap();
    dbg!(&res);
    dbg!(res.text().await.unwrap());
    StatusCode::OK
}
