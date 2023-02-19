pub mod user;

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}
