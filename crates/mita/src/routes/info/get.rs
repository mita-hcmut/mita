use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Extension,
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
        .post(
            state
                .config
                .moodle
                .url
                .join("webservice/rest/server.php")
                .unwrap(),
        )
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
    #[error("error getting moodle token")]
    GetMoodleToken(#[from] VaultError),
}

impl IntoResponse for InfoError {
    fn into_response(self) -> Response {
        let status = match &self {
            InfoError::GetMoodleToken(e) => e.status(),
        };
        tracing::error!(service = "vault", %status, error = ?self);
        status.into_response()
    }
}
