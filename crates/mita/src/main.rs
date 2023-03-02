use eyre::Context;
use mita::entrypoint::build;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    build().await.wrap_err("error trying to start server")?;

    Ok(())
}
