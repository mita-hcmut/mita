use eyre::Context;
use mita::{config::Config, entrypoint::Server};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::dev().wrap_err("error reading config")?.leak();

    Server::build(config)
        .await
        .wrap_err("error trying to build server")?
        .await
        .wrap_err("error trying to run server")?;

    Ok(())
}
