use axum::{extract::State, Form};
use axum_auth::AuthBearer;
use reqwest::StatusCode;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

use crate::app_state::AppState;

#[derive(Deserialize)]
pub struct VaultLoginResponse {
    pub auth: VaultLoginResponseAuth,
}

#[derive(Deserialize)]
pub struct VaultLoginResponseAuth {
    pub client_token: Secret<String>,
    pub entity_id: Secret<String>,
}

#[derive(Deserialize)]
pub struct FormData {
    moodle_token: Secret<String>,
}

pub async fn register_token(
    AuthBearer(token): AuthBearer,
    state: State<AppState>,
    Form(form): Form<FormData>,
) -> StatusCode {
    let res = state
        .0
        .http_client
        .post(format!("{}/v1/auth/jwt/login", state.0.config.vault.url))
        .json(&serde_json::json!({
            "role": "user",
            "jwt": &token,
        }))
        .send()
        .await
        .unwrap();

    let res: VaultLoginResponse = res.json().await.unwrap();

    let res = state
        .0
        .http_client
        .post(format!(
            "{}/v1/secret/data/{}",
            state.0.config.vault.url,
            res.auth.entity_id.expose_secret(),
        ))
        .header("X-Vault-Token", res.auth.client_token.expose_secret())
        .json(&serde_json::json!({
            "data": {
                "moodle_token": &form.moodle_token.expose_secret(),
            }
        }))
        .send()
        .await
        .unwrap();

    dbg!(&res);
    dbg!(res.text().await.unwrap());

    StatusCode::OK
}
