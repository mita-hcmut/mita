use axum::{extract::State, Form, Extension};
use axum_auth::AuthBearer;
use reqwest::StatusCode;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

use crate::{app_state::AppState, middlewares::vault::{ClientToken, EntityId}};

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
    client_token: Extension<ClientToken>,
    entity_id: Extension<EntityId>,
    state: State<AppState>,
    Form(form): Form<FormData>,
) -> StatusCode {
    let res = state
        .0
        .http_client
        .post(format!(
            "{}/v1/secret/data/{}",
            state.0.config.vault.url,
            entity_id.0.0.expose_secret(),
        ))
        .header("X-Vault-Token", client_token.0.0.expose_secret())
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
