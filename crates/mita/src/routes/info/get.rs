use axum::extract::State;
use axum_auth::AuthBearer;
use reqwest::StatusCode;
use secrecy::ExposeSecret;

use crate::{app_state::AppState, routes::token::put::VaultLoginResponse};

pub async fn get_info(id_token: AuthBearer, state: State<AppState>) -> StatusCode {
    // get moodle token from vault
    let res = state
        .0
        .http_client
        .post(format!("{}/v1/auth/jwt/login", state.0.config.vault.url))
        .json(&serde_json::json!({
            "role": "user",
            "jwt": &id_token.0,
        }))
        .send()
        .await
        .unwrap();

    let res: VaultLoginResponse = res.json().await.unwrap();

    let res = state
        .0
        .http_client
        .get(format!(
            "{}/v1/secret/data/{}",
            state.0.config.vault.url,
            res.auth.entity_id.expose_secret(),
        ))
        .header("X-Vault-Token", res.auth.client_token.expose_secret())
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
