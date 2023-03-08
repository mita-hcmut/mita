use async_trait::async_trait;
use eyre::Context;
use mita_config::VaultConfig;
use reqwest::StatusCode;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use thiserror::Error;
use tracing::{info_span, Instrument};
use url::Url;

use crate::moodle::token::MoodleToken;

#[derive(Clone)]
pub struct Client {
    config: &'static VaultConfig,
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
    #[tracing::instrument(skip(http_client, config, id_token))]
    pub async fn login(
        http_client: &reqwest::Client,
        config: &'static VaultConfig,
        id_token: &str,
    ) -> Result<Self, VaultError> {
        let mut login_url = config.url.clone();
        login_url
            .path_segments_mut()
            .map_err(|_| eyre::eyre!("vault url not a base"))?
            .extend(["v1", "auth", &config.user_data_path, "login"]);

        let res = http_client
            .post(login_url)
            .json(&serde_json::json!({
                "role": "user",
                "jwt": &id_token,
            }))
            .send()
            .instrument(info_span!("logging into vault using jwt"))
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
            config,
            http_client: http_client.clone(),
            client_token: ClientToken(res.auth.client_token),
            entity_id: EntityId(res.auth.entity_id),
        })
    }

    #[tracing::instrument(skip(self, moodle_token))]
    pub async fn put_moodle_token(&self, moodle_token: &MoodleToken) -> Result<(), VaultError> {
        self.http_client
            .post(self.data_path()?)
            .header("X-Vault-Token", self.client_token.0.expose_secret())
            .json(&serde_json::json!({
                "data": {
                    "moodle_token": &moodle_token.expose_secret(),
                }
            }))
            .send()
            .instrument(info_span!("putting moodle token in vault"))
            .await
            .wrap_err("error putting token in vault")?
            .try_into_vault_error()
            .await?;

        Ok(())
    }

    pub fn data_path(&self) -> Result<Url, VaultError> {
        let mut url = self.config.url.clone();
        url.path_segments_mut()
            .map_err(|_| eyre::eyre!("vault url not a base"))?
            .extend([
                "v1",
                &self.config.user_data_path,
                "data",
                &self.config.user_data_version,
                self.entity_id.0.expose_secret(),
            ]);
        Ok(dbg!(url))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_moodle_token(&self) -> Result<MoodleToken, VaultError> {
        let res = self
            .http_client
            .get(self.data_path()?)
            .header("X-Vault-Token", self.client_token.0.expose_secret())
            .send()
            .instrument(info_span!("getting moodle token from vault"))
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

        Ok(res
            .data
            .data
            .moodle_token
            .expose_secret()
            .parse()
            .wrap_err("malformed token inside vault")?)
    }
}

#[async_trait]
trait TryIntoVaultError: Sized {
    async fn try_into_vault_error(self) -> Result<Self, VaultError>;
}

#[async_trait]
impl TryIntoVaultError for reqwest::Response {
    async fn try_into_vault_error(self) -> Result<Self, VaultError> {
        if self.error_for_status_ref().is_ok() {
            return Ok(self);
        }

        let status = self.status();

        #[derive(Deserialize)]
        struct Body {
            #[serde(default)]
            errors: Vec<String>,
            #[serde(default)]
            warnings: Vec<String>,
        }

        let mut body: Body = self
            .json()
            .await
            .wrap_err("error deserializing error body")?;

        body.errors.extend(body.warnings);

        Err(VaultError::Status(status, body.errors))
    }
}

impl VaultError {
    pub fn status(&self) -> StatusCode {
        match self {
            &Self::Status(status, _) => status,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<eyre::Error> for VaultError {
    fn from(v: eyre::Error) -> Self {
        Self::Unexpected(v)
    }
}
