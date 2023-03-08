use axum::{extract::State, response::IntoResponse, response::Response, Extension, Form};
use reqwest::StatusCode;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use thiserror::Error;
use mita_moodle::error::MoodleError;
use mita_vault::VaultError;

use crate::app_state::AppState;

#[derive(Deserialize)]
pub struct FormData {
    moodle_token: Secret<String>,
}

#[axum::debug_handler]
#[tracing::instrument(skip(vault, state, form))]
pub async fn register_token(
    vault: Extension<mita_vault::Client>,
    state: State<AppState>,
    form: Form<FormData>,
) -> Result<StatusCode, RegisterError> {
    let moodle_token = form
        .moodle_token
        .expose_secret()
        .parse()
        .map_err(RegisterError::ValidateToken)?;

    // verify token by making a request to moodle
    let moodle =
        mita_moodle::Client::new(&state.http_client, &state.config.moodle, moodle_token).await?;

    vault.put_moodle_token(moodle.token()).await?;

    Ok(StatusCode::OK)
}

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("error putting moodle token")]
    PutMoodleToken(#[from] VaultError),
    #[error("error validating token")]
    ValidateToken(#[source] eyre::Error),
    #[error("error verifying token")]
    VerifyToken(#[from] MoodleError),
}

impl IntoResponse for RegisterError {
    fn into_response(self) -> Response {
        let status = match &self {
            RegisterError::PutMoodleToken(e) => e.status(),
            RegisterError::ValidateToken(_) => StatusCode::BAD_REQUEST,
            RegisterError::VerifyToken(e) => e.status(),
        };
        let service = match &self {
            RegisterError::PutMoodleToken(_) => "vault",
            RegisterError::ValidateToken(_) => "mita",
            RegisterError::VerifyToken(_) => "moodle",
        };
        tracing::error!(%service, %status, error = ?self);
        status.into_response()
    }
}
