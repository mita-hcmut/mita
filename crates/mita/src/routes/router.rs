use axum::{
    middleware,
    routing::{get, put},
    Router,
};

use super::{info::get::get_info, root, token::put::register_token};
use crate::{
    app_state::AppState,
    middlewares::{moodle::build_moodle_client, vault::authenticate},
    telemetry::router_telemetry_layer,
};

pub fn app_router(state: AppState) -> Router<()> {
    Router::new()
        .route("/", get(root))
        .merge(protected_router(state.clone()))
        .with_state(state)
        .layer(router_telemetry_layer())
}

fn protected_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/token", put(register_token))
        .merge(registered_router(state.clone()))
        .layer(middleware::from_fn_with_state(state, authenticate))
}

fn registered_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/info", get(get_info))
        .layer(middleware::from_fn_with_state(state, build_moodle_client))
}
