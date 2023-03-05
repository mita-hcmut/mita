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
use tracing_chrome::{ChromeLayerBuilder, FlushGuard};
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt::format,
    prelude::*,
};

pub fn setup() -> FlushGuard {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .expect("error parsing tracing filter");

    let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();

    tracing_subscriber::registry()
        .with(chrome_layer)
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_span_events(format::FmtSpan::FULL))
        .init();

    _guard
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
