pub mod router;
pub mod token;

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}
