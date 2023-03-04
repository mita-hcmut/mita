use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum_auth::AuthBearer;
use eyre::Context;
use reqwest::StatusCode;
use secrecy::Secret;
use serde::Deserialize;
use thiserror::Error;

use crate::app_state::AppState;

pub async fn authenticate<B>(
    state: State<AppState>,
    id_token: AuthBearer,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, AuthError> {
    let response = state
        .0
        .http_client
        .post(format!("{}/v1/auth/jwt/login", state.0.config.vault.url))
        .json(&serde_json::json!({
            "role": "user",
            "jwt": &id_token.0,
        }))
        .send()
        .await
        .wrap_err("error sending request to vault")?;

    if let Err(e) = response.error_for_status_ref() {
        let status = e.status().unwrap();

        #[derive(Deserialize)]
        struct ErrorResponse {
            errors: Vec<String>,
        }

        let body: ErrorResponse = response.json().await.wrap_err("error parsing response")?;

        tracing::error!("vault return status {}", status);
        tracing::error!("vault return errors: {:?}", body.errors);

        let e = match status {
            StatusCode::FORBIDDEN => AuthError::Unauthorized,
            StatusCode::BAD_REQUEST => AuthError::Validation(body.errors),
            _ => eyre::eyre!("unknown error response from vault").into(),
        };
        return Err(e);
    }

    let res: VaultLoginResponse = response.json().await.wrap_err("error parsing response")?;

    req.extensions_mut()
        .insert(ClientToken(res.auth.client_token));

    req.extensions_mut().insert(EntityId(res.auth.entity_id));

    Ok(next.run(req).await)
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("unexpected error: {0}")]
    Unexpected(#[source] eyre::Error),
    #[error("token malformed")]
    Validation(Vec<String>),
    #[error("unauthorized token")]
    Unauthorized,
}

#[derive(Deserialize)]
struct VaultLoginResponse {
    auth: VaultLoginResponseAuth,
}

#[derive(Deserialize)]
struct VaultLoginResponseAuth {
    client_token: Secret<String>,
    entity_id: Secret<String>,
}

#[derive(Clone, Deserialize)]
pub struct ClientToken(pub Secret<String>);

#[derive(Clone, Deserialize)]
pub struct EntityId(pub Secret<String>);

impl From<eyre::Error> for AuthError {
    fn from(v: eyre::Error) -> Self {
        Self::Unexpected(v)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            AuthError::Unexpected(e) => {
                tracing::error!("unexpected error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "unexpected error").into_response()
            }
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "invalid token").into_response(),
            AuthError::Validation(e) => (StatusCode::BAD_REQUEST, Json(e)).into_response(),
        }
    }
}
