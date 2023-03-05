use axum::{response::IntoResponse, response::Response, Extension, Form, Json};
use reqwest::StatusCode;
use secrecy::Secret;
use serde::Deserialize;
use thiserror::Error;

use crate::vault::{self, VaultError};

#[derive(Deserialize)]
pub struct FormData {
    moodle_token: Secret<String>,
}

#[tracing::instrument(skip(vault, form))]
pub async fn register_token(
    vault: Extension<vault::Client>,
    form: Form<FormData>,
) -> Result<StatusCode, RegisterError> {
    vault.put_moodle_token(&form.moodle_token).await?;

    Ok(StatusCode::OK)
}

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("unexpected error")]
    Unexpected(#[from] eyre::Error),
}

impl From<VaultError> for RegisterError {
    fn from(e: VaultError) -> Self {
        RegisterError::Unexpected(e.into())
    }
}

impl IntoResponse for RegisterError {
    fn into_response(self) -> Response {
        let errors = match &self {
            RegisterError::Unexpected(e) => vec![e.to_string()],
        };
        let status = match self {
            RegisterError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        tracing::error!(%status, ?errors);
        (status, Json(serde_json::json!({ "errors": errors }))).into_response()
    }
}
