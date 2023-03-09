use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MoodleError {
    #[error("unexpected error")]
    Unexpected(#[from] eyre::Error),
    #[error("error from moodle api")]
    Api(#[from] MoodleApiError),
}

#[derive(Error, Debug, Deserialize)]
#[error("error from moodle api: {message}")]
pub struct MoodleApiError {
    #[serde(rename = "errorcode")]
    pub kind: MoodleApiErrorKind,
    pub message: String,
}

#[derive(Debug, serde_enum_str::Deserialize_enum_str)]
#[serde(rename_all = "lowercase")]
pub enum MoodleApiErrorKind {
    InvalidToken,
    #[serde(other)]
    Unknown(String),
}

impl MoodleError {
    pub fn status(&self) -> StatusCode {
        match self {
            MoodleError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MoodleError::Api(e) => match e.kind {
                MoodleApiErrorKind::InvalidToken => StatusCode::UNAUTHORIZED,
                MoodleApiErrorKind::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }
}
