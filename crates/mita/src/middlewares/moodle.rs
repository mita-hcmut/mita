use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};
use reqwest::StatusCode;
use thiserror::Error;

use crate::app_state::AppState;
use mita_moodle::{self, error::MoodleError};
use mita_vault::{self, VaultError};

#[tracing::instrument(skip(vault, state, req, next))]
pub async fn build_moodle_client<B>(
    vault: Extension<mita_vault::Client>,
    state: State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, BuildMoodleError> {
    let token = vault.get_moodle_token().await?;

    let moodle = mita_moodle::Client::new(&state.http_client, &state.config.moodle, token).await?;

    req.extensions_mut().insert(moodle);

    Ok(next.run(req).await)
}

#[derive(Error, Debug)]
pub enum BuildMoodleError {
    #[error("error getting moodle token from vault")]
    GetToken(#[from] VaultError),
    #[error("error building moodle client using token")]
    BuildClient(#[from] MoodleError),
}

impl IntoResponse for BuildMoodleError {
    fn into_response(self) -> Response {
        let status = match &self {
            BuildMoodleError::GetToken(e) => e.status(),
            BuildMoodleError::BuildClient(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let service = match &self {
            BuildMoodleError::GetToken(_) => "vault",
            BuildMoodleError::BuildClient(_) => "moodle",
        };
        tracing::error!(%service, %status, error = ?self);
        status.into_response()
    }
}
