use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Extension, Json,
};
use reqwest::{header, StatusCode};
use secrecy::ExposeSecret;
use thiserror::Error;

use crate::{
    app_state::AppState,
    vault::{self, VaultError},
};

#[axum::debug_handler]
#[tracing::instrument(skip(vault, state))]
pub async fn get_info(
    vault: Extension<vault::Client>,
    state: State<AppState>,
) -> Result<Response, InfoError> {
    let res = vault.get_moodle_token().await?;

    // get info from moodle url
    let res = state
        .http_client
        .post(format!(
            "{}/webservice/rest/server.php",
            state.config.moodle.url
        ))
        .form(&[
            ("wstoken", res.expose_secret().as_str()),
            ("wsfunction", "core_webservice_get_site_info"),
            ("moodlewsrestformat", "json"),
        ])
        .send()
        .await
        .unwrap();

    let res = (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        res.bytes().await.unwrap(),
    )
        .into_response();

    Ok(res)
}

#[derive(Error, Debug)]
pub enum InfoError {
    #[error("unexpected error")]
    Unexpected(#[from] eyre::Error),
    #[error("token not found")]
    TokenNotFound,
}

impl From<VaultError> for InfoError {
    fn from(e: VaultError) -> Self {
        match e {
            VaultError::Unexpected(e) => InfoError::Unexpected(e),
            VaultError::Status(status, _) => {
                if status == StatusCode::NOT_FOUND {
                    return InfoError::TokenNotFound;
                }
                eyre::eyre!(e).into()
            }
        }
    }
}

impl IntoResponse for InfoError {
    fn into_response(self) -> Response {
        let errors = match &self {
            InfoError::Unexpected(e) => vec![e.to_string()],
            InfoError::TokenNotFound => vec![self.to_string()],
        };
        let status = match self {
            InfoError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
            InfoError::TokenNotFound => StatusCode::NOT_FOUND,
        };
        tracing::error!(%status, ?errors);
        (status, Json(serde_json::json!({ "errors": errors }))).into_response()
    }
}
