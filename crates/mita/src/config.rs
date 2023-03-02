use eyre::Context;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub app: AppConfig,
    pub database: DatabaseConfig,
    pub vault: VaultConfig,
    pub oauth2: Option<OAuth2Config>,
}

#[derive(Deserialize, Serialize)]
pub struct AppConfig {
    pub hostname: String,
    pub port: u16,
}

#[derive(Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Deserialize, Serialize)]
pub struct VaultConfig {
    pub url: String,
}

#[derive(Deserialize, Serialize)]
pub struct OAuth2Config {
    pub url: String,
    pub client_id: String,
}

impl Config {
    fn figment() -> Figment {
        Figment::new()
            .merge(Toml::file("App.toml").nested())
            .merge(Env::prefixed("APP_").split("__"))
    }

    pub fn dev() -> eyre::Result<Self> {
        Config::figment()
            .extract()
            .wrap_err("error reading dev config")
    }

    pub fn test() -> eyre::Result<Self> {
        Figment::from(Serialized::defaults(Config::dev()?))
            .merge(Config::figment().select("test"))
            .extract()
            .wrap_err("error reading test config")
    }
}