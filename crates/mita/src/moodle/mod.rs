pub mod error;
pub mod json_response;
pub mod token;

use eyre::WrapErr;
use secrecy::ExposeSecret;
use serde::Deserialize;
use tracing::{info_span, Instrument};

use crate::config::MoodleConfig;

use self::{error::MoodleError, json_response::MoodleJson, token::MoodleToken};

pub struct Client {
    pub http_client: reqwest::Client,
    pub config: &'static MoodleConfig,
    pub moodle_token: MoodleToken,
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
            .get(self.config.url.join("webservice/rest/server.php").unwrap())
            .query(&[
                ("wstoken", self.moodle_token.expose_secret().as_str()),
                ("wsfunction", "core_webservice_get_site_info"),
                ("moodlewsrestformat", "json"),
            ])
            .send()
            .instrument(info_span!("getting moodle info"))
            .await
            .wrap_err("error sending request to moodle")?;

        Ok(res
            .moodle_json()
            .await
            .wrap_err("error deserializing moodle response")?)
    }
}

#[derive(Debug, Deserialize)]
pub struct InfoResponse {
    pub fullname: String,
}
