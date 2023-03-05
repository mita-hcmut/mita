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
