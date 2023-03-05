use axum::{response::IntoResponse, response::Response, Extension, Form};
use reqwest::StatusCode;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use thiserror::Error;

use crate::vault::{self, VaultError};

#[derive(Deserialize)]
pub struct FormData {
    moodle_token: Secret<String>,
}

#[axum::debug_handler]
#[tracing::instrument(skip(vault, form))]
pub async fn register_token(
    vault: Extension<vault::Client>,
    form: Form<FormData>,
) -> Result<StatusCode, RegisterError> {
    let moodle_token = form
        .moodle_token
        .expose_secret()
        .parse()
        .map_err(RegisterError::ValidateToken)?;

    vault.put_moodle_token(&moodle_token).await?;

    Ok(StatusCode::OK)
}

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("error putting moodle token")]
    PutMoodleToken(#[from] VaultError),
    #[error("error validating token")]
    ValidateToken(#[source] eyre::Error),
}

impl IntoResponse for RegisterError {
    fn into_response(self) -> Response {
        let status = match &self {
            RegisterError::PutMoodleToken(e) => e.status(),
            RegisterError::ValidateToken(_) => StatusCode::BAD_REQUEST,
        };
        tracing::error!(service = "vault", %status, error = ?self);
        status.into_response()
    }
}
