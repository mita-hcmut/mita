use axum::{extract::State, Json};
use sqlx::SqlitePool;

pub async fn get_tokens(State(pool): State<SqlitePool>) -> Json<Vec<Vec<u8>>> {
    let rows = sqlx::query!("SELECT token from users")
        .fetch_all(&pool)
        .await
        .unwrap();
    Json(Vec::from_iter(rows.into_iter().map(|row| row.token)))
}
