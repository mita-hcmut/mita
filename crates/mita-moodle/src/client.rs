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

#[derive(Debug, Deserialize)]
pub struct InfoResponse {
    pub fullname: String,
}

#[derive(Debug, Deserialize)]
pub struct CoursesResponse {
    pub courses: Vec<Course>,
}

#[derive(Debug, Deserialize)]
pub struct Course {
    pub id: u32,
    pub fullname: String,
    #[serde(rename = "coursecategory")]
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct ContentResponse(pub Vec<Section>);

#[derive(Debug, Deserialize)]
pub struct Section {
    pub id: u32,
    pub name: String,
    pub modules: Vec<Module>,
}

#[derive(Debug, Deserialize)]
pub struct Module {
    pub id: u32,
    pub name: String,
    pub modname: String,
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

    #[tracing::instrument(skip(self))]
    pub async fn get_courses(&self) -> Result<CoursesResponse, MoodleError> {
        self.request(
            "core_courses_get_courses_by_classification",
            &[("classification", "inprogress")],
        )
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_content(&self, courseid: u32) -> Result<ContentResponse, MoodleError> {
        self.request(
            "core_courses_get_contents",
            &[("courseid", &courseid.to_string())],
        )
        .await
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
