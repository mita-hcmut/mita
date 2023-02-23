pub mod put_token;
pub mod get_tokens;

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}
