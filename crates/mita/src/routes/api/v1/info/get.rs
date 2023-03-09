use axum::{
    response::{IntoResponse, Response},
    Extension, Json,
};
use eyre::Context;
use reqwest::StatusCode;
use serde_json::json;
use thiserror::Error;

#[axum::debug_handler]
#[tracing::instrument(skip(moodle))]
pub async fn get_info(moodle: Extension<mita_moodle::Client>) -> Result<Response, InfoError> {
    // this should always succeed because the middleware should have already
    // verified the token. if it fails, the moodle server is in a bad state
    let info = moodle
        .get_info()
        .await
        .wrap_err("error getting info from moodle")?;

    Ok(Json(json!({ "fullname": &info.fullname })).into_response())
}

#[derive(Error, Debug)]
pub enum InfoError {
    #[error("unexpected error")]
    Unexpected(#[from] eyre::Error),
}

impl IntoResponse for InfoError {
    fn into_response(self) -> Response {
        let status = match &self {
            InfoError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        tracing::error!(service = "moodle", %status, error = ?self);
        status.into_response()
    }
}
