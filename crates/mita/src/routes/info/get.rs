use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Extension,
};
use reqwest::{header, StatusCode};
use secrecy::ExposeSecret;

use crate::{app_state::AppState, vault};

#[axum::debug_handler]
#[tracing::instrument(skip(vault, state))]
pub async fn get_info(vault: Extension<vault::Client>, state: State<AppState>) -> Response {
    let res = vault.get_moodle_token().await.unwrap();

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

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        res.bytes().await.unwrap(),
    )
        .into_response()
}
