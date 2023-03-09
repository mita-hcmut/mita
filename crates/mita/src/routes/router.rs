use axum::Router;

use crate::{app_state::AppState, telemetry::router_telemetry_layer};

use super::api::v1::router::api_router;

pub fn app_router(state: AppState) -> Router<()> {
    Router::new()
        .nest("/api/v1", api_router(state.clone()))
        .with_state(state)
        .layer(router_telemetry_layer())
}
