use axum::{body::Body, http::Request};
use tower::{
    layer::util::{Identity, Stack},
    ServiceBuilder,
};
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
    ServiceBuilderExt,
};
use tracing::Span;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    prelude::*,
};
use tracing_tree::HierarchicalLayer;

pub struct Guard;

pub fn setup() -> Guard {
    color_eyre::install().expect("error setting color_eyre as global panic handler");

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::OFF.into())
        .from_env()
        .expect("error parsing tracing filter");

    tracing_subscriber::registry()
        .with(env_filter)
        .with(HierarchicalLayer::default())
        .with(ErrorLayer::default())
        .init();

    Guard
}

// extract the type so that clippy wont yell at me
type RouterTelemetryLayer<MakeSpan> = Stack<
    PropagateRequestIdLayer,
    Stack<
        TraceLayer<SharedClassifier<ServerErrorsAsFailures>, MakeSpan>,
        Stack<SetRequestIdLayer<MakeRequestUuid>, Identity>,
    >,
>;

pub fn router_telemetry_layer() -> RouterTelemetryLayer<impl Fn(&Request<Body>) -> Span + Clone> {
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
        .into_inner()
}
