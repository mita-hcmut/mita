use std::sync::Arc;

use eyre::Context;
use mita::{config::Config, entrypoint::Server};
use proptest::{
    strategy::{Strategy, ValueTree},
    test_runner::TestRunner,
};

mod helper;

#[tokio::test]
pub async fn oauth2() -> eyre::Result<()> {
    let config = Config::test()?;
    let server = Server::build(Arc::new(config)).await?;
    let addr = server.addr();

    tokio::spawn(server);

    let res = helper::oauth2::get_code("khang", "").await;
    let client = reqwest::Client::new();
    let mut runner = TestRunner::default();

    let token = "[a-f0-9]{32}"
        .new_tree(&mut runner)
        .map_err(|e| eyre::eyre!(e))?
        .current();

    let res = client
        .put(format!("http://{addr}/token"))
        .bearer_auth(&res.id_token)
        .form(&[("moodle_token", &token)])
        .send()
        .await
        .wrap_err_with(|| format!("error putting token {token}"))?;

    dbg!(res);

    Ok(())
}
