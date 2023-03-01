mod helper;

#[tokio::test]
pub async fn oauth2() {
    let res = helper::oauth2::get_code("khang", "").await;
    let client = reqwest::Client::new();
    let res = client
        .put("http://localhost:3000/token")
        .bearer_auth(&res.id_token)
        .send()
        .await
        .unwrap();
    dbg!(res);
}
