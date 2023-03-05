use axum::{Extension, Form};
use reqwest::StatusCode;
use secrecy::Secret;
use serde::Deserialize;

use crate::vault;

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

#[tracing::instrument(skip(vault, form))]
pub async fn register_token(vault: Extension<vault::Client>, form: Form<FormData>) -> StatusCode {
    vault.put_moodle_token(&form.moodle_token).await.unwrap();

    StatusCode::OK
}
