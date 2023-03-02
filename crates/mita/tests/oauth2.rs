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

    let id_token = helper::oauth2::get_code("khang", "").await.id_token;
    let client = reqwest::Client::new();
    let mut runner = TestRunner::default();

    let token = "[a-f0-9]{32}"
        .new_tree(&mut runner)
        .map_err(|e| eyre::eyre!(e))?
        .current();

    client
        .put(format!("http://{addr}/token"))
        .bearer_auth(&id_token)
        .form(&[("moodle_token", &token)])
        .send()
        .await
        .wrap_err_with(|| format!("error putting token {token}"))?
        .error_for_status()?;

    let res = client
        .get(format!("http://{addr}/info"))
        .bearer_auth(&id_token)
        .send()
        .await
        .wrap_err("error getting user info")?
        .error_for_status()?;

    dbg!(&res);
    dbg!(res.text().await.unwrap());

    Ok(())
}
