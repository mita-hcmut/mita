use eyre::Context;
use serde::{de::DeserializeOwned, Deserialize};

use super::error::{MoodleApiError, MoodleError};

#[async_trait::async_trait]
pub trait MoodleJson {
    async fn moodle_json<T: DeserializeOwned>(self) -> Result<T, MoodleError>;
}

#[async_trait::async_trait]
impl MoodleJson for reqwest::Response {
    async fn moodle_json<T: DeserializeOwned>(self) -> Result<T, MoodleError> {
        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        pub enum MoodleApiResponse<R> {
            Err(MoodleApiError),
            Ok(R),
        }
        match self.json().await.wrap_err("error deserializing body")? {
            MoodleApiResponse::Ok(r) => Ok(r),
            MoodleApiResponse::Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

    use crate::moodle::{error::MoodleError, json_response::MoodleJson, InfoResponse};

    #[tokio::test]
    async fn deserialize_invalid_token() -> eyre::Result<()> {
        let mock = MockServer::start().await;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "errorcode": "invalidtoken",
                "exception": "moodle_exception",
                "message": "Invalid token - token expired"
            })))
            .expect(1)
            .mount(&mock)
            .await;

        let res = reqwest::get(&mock.uri())
            .await?
            .moodle_json::<InfoResponse>()
            .await;

        claims::assert_matches!(res, Err(MoodleError::Api(_)));

        Ok(())
    }

    #[tokio::test]
    async fn deserialize_full_name() -> eyre::Result<()> {
        let mock = MockServer::start().await;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "fullname": "hoho",
            })))
            .expect(1)
            .mount(&mock)
            .await;

        let res = reqwest::get(&mock.uri())
            .await?
            .moodle_json::<InfoResponse>()
            .await;

        claims::assert_matches!(res, Ok(InfoResponse { .. }));

        Ok(())
    }

    #[tokio::test]
    async fn doesnt_crash_on_unexpected_code() -> eyre::Result<()> {
        let mock = MockServer::start().await;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "errorcode": "ohsnap",
                "message": "oh snap",
            })))
            .expect(1)
            .mount(&mock)
            .await;

        let res = reqwest::get(&mock.uri())
            .await?
            .moodle_json::<InfoResponse>()
            .await;

        claims::assert_err!(res);

        Ok(())
    }
}
