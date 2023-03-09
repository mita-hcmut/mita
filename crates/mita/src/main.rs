use std::env;

use eyre::Context;
use mita::{entrypoint::Server, telemetry};
use mita_config::Config;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    telemetry::setup();

    let config = if env::var("RUST_ENV") == Ok("production".into()) {
        Config::production()
    } else {
        Config::dev()
    }
    .wrap_err("error reading config")?
    .leak();

    Server::build(config)
        .await
        .wrap_err("error trying to build server")?
        .await
        .wrap_err("error trying to run server")?;

    Ok(())
}
