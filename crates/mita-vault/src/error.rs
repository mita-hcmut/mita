use async_trait::async_trait;
use eyre::Context;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("unexpected error: {0}")]
    Unexpected(#[source] eyre::Error),
    #[error("status {0}, errors: {1:?}")]
    Status(StatusCode, Vec<String>),
}

#[async_trait]
pub trait TryIntoVaultError: Sized {
    async fn try_into_vault_error(self) -> Result<Self, VaultError>;
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
