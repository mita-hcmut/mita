use axum::{
    routing::{get, put},
    Router,
};

use super::{info::get::get_info, root, token::put::register_token};
use crate::{app_state::AppState, middlewares::vault::authenticate};

pub fn app_router(state: AppState) -> Router<()> {
    Router::new()
        .route("/", get(root))
        .merge(protected_router(state.clone()))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

fn protected_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/token", put(register_token))
        .route("/info", get(get_info))
        .layer(axum::middleware::from_fn_with_state(state, authenticate))
}
