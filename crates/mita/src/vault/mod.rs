use async_trait::async_trait;
use eyre::Context;
use reqwest::{Response, StatusCode};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use thiserror::Error;
use tracing::Instrument;

#[derive(Clone)]
pub struct Client {
    url: &'static str,
    http_client: reqwest::Client,
    client_token: ClientToken,
    entity_id: EntityId,
}

#[derive(Clone, Deserialize)]
struct ClientToken(pub Secret<String>);

#[derive(Clone, Deserialize)]
struct EntityId(pub Secret<String>);

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("unexpected error: {0}")]
    Unexpected(#[source] eyre::Error),
    #[error("status {0}, errors: {1:?}")]
    Status(StatusCode, Vec<String>),
}

impl Client {
    #[tracing::instrument(skip(http_client, url, id_token))]
    pub async fn login(
        http_client: &reqwest::Client,
        url: &'static str,
        id_token: &str,
    ) -> Result<Self, VaultError> {
        let res = http_client
            .post(format!("{}/v1/auth/jwt/login", url))
            .json(&serde_json::json!({
                "role": "user",
                "jwt": &id_token,
            }))
            .send()
            .instrument(tracing::info_span!("sending request to vault"))
            .await
            .wrap_err("error sending request to vault")?
            .try_into_vault_error()
            .await?;

        #[derive(Deserialize)]
        struct Response {
            auth: ResponseAuth,
        }

        #[derive(Deserialize)]
        struct ResponseAuth {
            client_token: Secret<String>,
            entity_id: Secret<String>,
        }

        let res: Response = res.json().await.wrap_err("could not read body as json")?;

        Ok(Self {
            url,
            http_client: http_client.clone(),
            client_token: ClientToken(res.auth.client_token),
            entity_id: EntityId(res.auth.entity_id),
        })
    }

    #[tracing::instrument(skip(self, moodle_token))]
    pub async fn put_moodle_token(&self, moodle_token: &Secret<String>) -> Result<(), VaultError> {
        self.http_client
            .post(format!(
                "{}/v1/secret/data/{}",
                self.url,
                self.entity_id.0.expose_secret(),
            ))
            .header("X-Vault-Token", self.client_token.0.expose_secret())
            .json(&serde_json::json!({
                "data": {
                    "moodle_token": &moodle_token.expose_secret(),
                }
            }))
            .send()
            .await
            .wrap_err("error putting token in vault")?
            .try_into_vault_error()
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_moodle_token(&self) -> Result<Secret<String>, VaultError> {
        let res = self
            .http_client
            .get(format!(
                "{}/v1/secret/data/{}",
                self.url,
                self.entity_id.0.expose_secret(),
            ))
            .header("X-Vault-Token", self.client_token.0.expose_secret())
            .send()
            .await
            .wrap_err("error getting token from vault")?
            .try_into_vault_error()
            .await?;

        #[derive(Deserialize)]
        struct Response {
            data: ResponseData,
        }

        #[derive(Deserialize)]
        struct ResponseData {
            data: ResponseDataData,
        }

        #[derive(Deserialize)]
        struct ResponseDataData {
            moodle_token: Secret<String>,
        }

        let res: Response = res.json().await.wrap_err("could not read body as json")?;

        Ok(res.data.data.moodle_token)
    }
}

#[async_trait]
trait TryIntoVaultError: Sized {
    async fn try_into_vault_error(self) -> Result<Self, VaultError>;
}

#[async_trait]
impl TryIntoVaultError for Response {
    async fn try_into_vault_error(self) -> Result<Self, VaultError> {
        if self.error_for_status_ref().is_ok() {
            return Ok(self);
        }

        let status = self.status();

        #[derive(Deserialize)]
        struct Body {
            errors: Vec<String>,
        }

        let body: Body = self
            .json()
            .await
            .wrap_err("error deserializing error body")?;

        Err(VaultError::Status(status, body.errors))
    }
}

impl From<eyre::Error> for VaultError {
    fn from(v: eyre::Error) -> Self {
        Self::Unexpected(v)
    }
}
