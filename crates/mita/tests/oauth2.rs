use eyre::Context;
use fake::Fake;
use mita::{config::Config, entrypoint::Server, telemetry};
use proptest::{
    strategy::{Strategy, ValueTree},
    test_runner::TestRunner,
};
use serde_json::{json, Value};
use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

mod helper;

#[tokio::test]
pub async fn oauth2() -> eyre::Result<()> {
    let _guard = telemetry::setup();

    let mut runner = TestRunner::default();
    let token = "[a-f0-9]{32}"
        .new_tree(&mut runner)
        .map_err(|e| eyre::eyre!(e))?
        .current();

    let mock_server = MockServer::start().await;
    let fullname = fake::faker::name::en::Name().fake::<String>();
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/webservice/rest/server.php"))
        .and(matchers::header(
            "Content-Type",
            "application/x-www-form-urlencoded",
        ))
        .and(matchers::body_string_contains(format!("wstoken={token}")))
        .and(matchers::body_string_contains(
            "wsfunction=core_webservice_get_site_info",
        ))
        .and(matchers::body_string_contains("moodlewsrestformat=json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&json!({
            "fullname": fullname,
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let mut config = Config::test()?;
    config.moodle.url = format!("http://{}", mock_server.address());
    let server = Server::build(config.leak()).await?;

    let addr = server.addr();

    tokio::spawn(server);

    let id_token = helper::oauth2::get_code("khang", "").await.id_token;
    let client = reqwest::Client::new();

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

    let body: Value = res.json().await?;

    assert_eq!(body["fullname"], fullname);

    Ok(())
}

#[tokio::test]
async fn shoud_error_when_no_token_provided() -> eyre::Result<()> {
    telemetry::setup();

    let config = Config::test()?;
    let server = Server::build(config.leak()).await?;

    let addr = server.addr();

    tokio::spawn(server);

    let client = reqwest::Client::new();

    let res = client
        .get(format!("http://{addr}/info"))
        .bearer_auth("invalid_token")
        .send()
        .await
        .wrap_err("error getting user info")?;

    assert_eq!(res.status(), 400);

    let res = client
        .get(format!("http://{addr}/info"))
        .bearer_auth("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c")
        .send()
        .await
        .wrap_err("error getting user info")?;

    assert_eq!(res.status(), 400);

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    Ok(())
}
