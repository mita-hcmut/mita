pub mod error;
pub mod json_response;
pub mod token;

use eyre::WrapErr;
use secrecy::ExposeSecret;
use serde::Deserialize;
use tracing::{info_span, Instrument};

use mita_config::MoodleConfig;

use self::{error::MoodleError, json_response::MoodleJson, token::MoodleToken};

#[derive(Clone)]
pub struct Client {
    http_client: reqwest::Client,
    config: &'static MoodleConfig,
    moodle_token: MoodleToken,
}

impl Client {
    #[tracing::instrument(skip(http_client, config, moodle_token))]
    pub async fn new(
        http_client: &reqwest::Client,
        config: &'static MoodleConfig,
        moodle_token: MoodleToken,
    ) -> Result<Self, MoodleError> {
        let client = Self {
            http_client: http_client.clone(),
            config,
            moodle_token,
        };

        // validate token by sending a request to moodle
        client.get_info().await?;

        Ok(client)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_info(&self) -> Result<InfoResponse, MoodleError> {
        let res = self
            .http_client
            .post(self.url()?)
            .form(&[
                ("wstoken", self.moodle_token.expose_secret().as_str()),
                ("wsfunction", "core_webservice_get_site_info"),
                ("moodlewsrestformat", "json"),
            ])
            .send()
            .instrument(info_span!("getting moodle info"))
            .await
            .wrap_err("error sending request to moodle")?;

        res.moodle_json().await
    }

    pub fn token(&self) -> &MoodleToken {
        &self.moodle_token
    }

    pub fn url(&self) -> eyre::Result<url::Url> {
        self.config
            .url
            .join("webservice/rest/server.php")
            .wrap_err("invalid moodle url")
    }
}

#[derive(Debug, Deserialize)]
pub struct InfoResponse {
    pub fullname: String,
}
