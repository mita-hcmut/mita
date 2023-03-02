use axum::{
    routing::{get, put},
    Router,
};

use crate::app_state::AppState;

use super::{info::get::get_info, root, token::put::register_token};

pub fn app_router() -> Router<AppState> {
    Router::new()
        .route("/", get(root))
        .route("/token", put(register_token))
        .route("/info", get(get_info))
}
