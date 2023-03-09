use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_auth::AuthBearer;
use thiserror::Error;

use crate::app_state::AppState;
use mita_vault::VaultError;

#[tracing::instrument(skip(state, id_token, req, next))]
pub async fn authenticate<B>(
    state: State<AppState>,
    id_token: AuthBearer,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, AuthError> {
    let vault =
        mita_vault::Client::login(&state.http_client, &state.config.vault, &id_token.0).await?;
    req.extensions_mut().insert(vault);
    Ok(next.run(req).await)
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct AuthError(#[from] VaultError);

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = self.0.status();
        tracing::error!(service = "vault", %status, error = ?self, "error logging into vault");
        status.into_response()
    }
}
