use mita::{config::Config, entrypoint::Server};

mod helper;

#[tokio::test]
pub async fn oauth2() -> eyre::Result<()> {
    let config = Config::test()?;
    let server = Server::build(&config).await?;
    let addr = server.addr();

    tokio::spawn(server);

    let res = helper::oauth2::get_code("khang", "").await;
    let client = reqwest::Client::new();
    let res = client
        .put(format!("http://{addr}/token"))
        .bearer_auth(&res.id_token)
        .send()
        .await?;

    dbg!(res);

    Ok(())
}
