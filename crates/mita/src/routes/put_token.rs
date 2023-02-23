use axum::{extract::State, Form};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::domain::moodle_token::MoodleToken;

#[derive(Deserialize)]
pub struct FormData {
    token: String,
}

pub async fn put_token(State(pool): State<SqlitePool>, Form(data): Form<FormData>) -> &'static str {
    let token: MoodleToken = data.token.parse().unwrap();
    let blob = token.as_ref();
    sqlx::query!("INSERT INTO users (token) VALUES ($1)", blob)
        .execute(&pool)
        .await
        .unwrap();
    "Hello, World!"
}
