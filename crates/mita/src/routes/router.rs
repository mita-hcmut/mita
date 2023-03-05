use axum::{
    body::Body,
    http::Request,
    routing::{get, put},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{request_id::MakeRequestUuid, trace::TraceLayer, ServiceBuilderExt};

use super::{info::get::get_info, root, token::put::register_token};
use crate::{app_state::AppState, middlewares::vault::authenticate};

pub fn app_router(state: AppState) -> Router<()> {
    Router::new()
        .route("/", get(root))
        .merge(protected_router(state.clone()))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .layer(
                    TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                        let request_id = request
                            .headers()
                            .get("x-request-id")
                            .expect("x-request-id header not set")
                            .to_str()
                            .expect("x-request-id header not valid ascii");

                        tracing::info_span!(
                            "request",
                            id = %request_id,
                            method = %request.method(),
                            uri = %request.uri(),
                        )
                    }),
                )
                .propagate_x_request_id()
                .into_inner(),
        )
}

fn protected_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/token", put(register_token))
        .route("/info", get(get_info))
        .layer(axum::middleware::from_fn_with_state(state, authenticate))
}
