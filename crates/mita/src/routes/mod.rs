pub mod router;
pub mod token;
pub mod info;

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}
