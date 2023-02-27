mod helper;

#[tokio::test]
pub async fn oauth2() {
    helper::oauth2::get_code().await;
}
