pub mod token;

use crate::config::MoodleConfig;

pub struct Client {
    pub http_client: reqwest::Client,
    pub config: &'static MoodleConfig,
    pub moodle_token: String,
}
