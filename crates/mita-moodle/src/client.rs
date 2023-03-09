use eyre::WrapErr;
use secrecy::ExposeSecret;
use serde::{de::DeserializeOwned, Deserialize};

use mita_config::MoodleConfig;
use tracing::Instrument;

use crate::{error::MoodleError, json_response::MoodleJson, token::MoodleToken};

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
    async fn request<R: DeserializeOwned>(
        &self,
        wsfunction: &str,
        params: &[(&str, &str)],
    ) -> Result<R, MoodleError> {
        let mut params = Vec::from(params);
        params.push(("wsfunction", wsfunction));
        params.push(("moodlewsrestformat", "json"));

        let res = self
            .http_client
            .post(self.url()?)
            // move non-sensitive information to query for
            // better logging opportunities
            .query(&params)
            .form(&[("wstoken", self.moodle_token.expose_secret().as_str())])
            .send()
            .instrument(tracing::info_span!("sending http request"))
            .await
            .wrap_err("error sending request to moodle")?;

        res.moodle_json().await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_info(&self) -> Result<InfoResponse, MoodleError> {
        self.request("core_webservice_get_site_info", &[]).await
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
