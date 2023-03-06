use eyre::Context;
use fake::{faker::internet::en::SafeEmail, Fake};
use jsonwebtoken::EncodingKey;
use proptest::{
    strategy::{Strategy, ValueTree},
    test_runner::TestRunner,
};
use serde_json::{json, Value};
use wiremock::{
    matchers::{self, any},
    Mock, ResponseTemplate,
};

use crate::helper::test_app::TestApp;

mod helper;

#[tokio::test]
pub async fn put_token_and_get_info_successfully() -> eyre::Result<()> {
    let mut app = TestApp::new().await?;

    let mut runner = TestRunner::default();
    let token = "[a-f0-9]{32}"
        .new_tree(&mut runner)
        .map_err(|e| eyre::eyre!(e))?
        .current();

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
        // TODO: test if api only called once
        .mount(&app.moodle_server)
        .await;

    app.id_token = helper::oauth2::get_code("khang", "").await.id_token;
    app.put_token(token)
        .await?
        .error_for_status()
        .wrap_err("got error status")?;

    // get another token with the same username
    app.id_token = helper::oauth2::get_code("khang", "").await.id_token;
    let res = app.get_info().await?.error_for_status()?;

    let body: Value = res.json().await?;

    assert_eq!(body["fullname"], fullname);

    Ok(())
}

#[tokio::test]
async fn shoud_400_when_no_token_provided() -> eyre::Result<()> {
    let app = TestApp::new().await?;
    let res = app.get_info().await?;
    assert_eq!(res.status(), 400);

    Ok(())
}

#[tokio::test]
async fn shoud_400_when_invalid_token_provided() -> eyre::Result<()> {
    let mut app = TestApp::new().await?;
    app.id_token = jsonwebtoken::encode(
        &Default::default(),
        &json!({
            "sub": SafeEmail().fake::<String>(),
        }),
        &EncodingKey::from_secret(b"secret"),
    )
    .unwrap();

    Mock::given(matchers::any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.moodle_server)
        .await;

    let res = app.get_info().await?;

    assert_eq!(res.status(), 400);

    Ok(())
}

#[tokio::test]
async fn shoud_404_when_unregistered_token_provided() -> eyre::Result<()> {
    let mut app = TestApp::new().await?;
    app.id_token = helper::oauth2::get_code("khang", "").await.id_token;
    let res = app.get_info().await?;

    assert_eq!(res.status(), 404);

    Ok(())
}

#[tokio::test]
async fn should_400_when_no_bearer_header() -> eyre::Result<()> {
    let app = TestApp::new().await?;
    let res = app.put_token_without_bearer("???".into()).await?;
    assert_eq!(res.status(), 400);
    Ok(())
}

#[tokio::test]
pub async fn should_400_when_moodle_token_incorrect_length() -> eyre::Result<()> {
    let mut app = TestApp::new().await?;
    app.id_token = helper::oauth2::get_code("khang", "").await.id_token;

    let mut runner = TestRunner::default();
    let token = "[a-f0-9]{28}"
        .new_tree(&mut runner)
        .map_err(|e| eyre::eyre!(e))?
        .current();

    let res = app.put_token(token).await?;

    assert_eq!(res.status(), 400);

    Ok(())
}

#[tokio::test]
pub async fn should_401_when_unknown_token() -> eyre::Result<()> {
    let mut app = TestApp::new().await?;

    let mut runner = TestRunner::default();
    let token = "[a-f0-9]{32}"
        .new_tree(&mut runner)
        .map_err(|e| eyre::eyre!(e))?
        .current();

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200).set_body_json(&json!({
            "errorcode": "invalidtoken",
            "exception": "moodle_exception",
            "message": "Invalid token - token not found",
        })))
        .expect(1)
        .mount(&app.moodle_server)
        .await;

    app.id_token = helper::oauth2::get_code("khang", "").await.id_token;

    let res = app.put_token(token).await?;

    assert_eq!(res.status(), 401);

    Ok(())
}
