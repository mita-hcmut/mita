use axum::{extract::State, Extension};
use reqwest::StatusCode;
use secrecy::ExposeSecret;

use crate::{
    app_state::AppState,
    middlewares::vault::{ClientToken, EntityId},
};

#[axum::debug_handler]
pub async fn get_info(
    client_token: Extension<ClientToken>,
    entity_id: Extension<EntityId>,
    state: State<AppState>,
) -> StatusCode {
    let res = state
        .0
        .http_client
        .get(format!(
            "{}/v1/secret/data/{}",
            state.0.config.vault.url,
            entity_id.0 .0.expose_secret(),
        ))
        .header("X-Vault-Token", client_token.0 .0.expose_secret())
        .send()
        .await
        .unwrap();

    // extract moodle token from vault response

    let res = res.json::<serde_json::Value>().await.unwrap();

    let moodle_token = res["data"]["data"]["moodle_token"].as_str().unwrap();

    // get info from moodle url
    let res = state
        .0
        .http_client
        .post(format!(
            "{}/webservice/rest/server.php",
            state.0.config.moodle.url
        ))
        .form(&[
            ("wstoken", moodle_token),
            ("wsfunction", "core_webservice_get_site_info"),
            ("moodlewsrestformat", "json"),
        ])
        .send()
        .await
        .unwrap();

    dbg!(&res);
    dbg!(res.text().await.unwrap());

    StatusCode::OK
}
