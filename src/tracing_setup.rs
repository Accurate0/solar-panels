use http::{HeaderMap, HeaderValue};
use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::BatchConfigBuilder,
    trace::{BatchSpanProcessor, Tracer},
};
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_NAME, TELEMETRY_SDK_LANGUAGE, TELEMETRY_SDK_NAME,
    TELEMETRY_SDK_VERSION,
};
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

pub fn external_tracer() -> Tracer {
    let ingest_url = std::env::var("OTEL_TRACING_URL").unwrap();

    let mut headers = HeaderMap::<HeaderValue>::with_capacity(1);
    headers.insert(
        "User-Agent",
        HeaderValue::from_str(&format!("solar-panels/{}", env!("CARGO_PKG_VERSION"))).unwrap(),
    );

    let tags = vec![
        KeyValue::new(TELEMETRY_SDK_NAME, "otel-tracing-rs".to_string()),
        KeyValue::new(TELEMETRY_SDK_VERSION, env!("CARGO_PKG_VERSION").to_string()),
        KeyValue::new(TELEMETRY_SDK_LANGUAGE, "rust".to_string()),
        KeyValue::new(SERVICE_NAME, format!("solar-panels")),
        KeyValue::new(
            DEPLOYMENT_ENVIRONMENT_NAME,
            if cfg!(debug_assertions) {
                "development"
            } else {
                "production"
            },
        ),
    ];

    let resource = Resource::builder_empty().with_attributes(tags).build();

    let batch_config = BatchConfigBuilder::default()
        .with_max_queue_size(20480)
        .build();

    let span_exporter = opentelemetry_otlp::HttpExporterBuilder::default()
        .with_protocol(Protocol::HttpJson)
        .with_endpoint(ingest_url)
        .with_timeout(Duration::from_secs(3))
        .build_span_exporter()
        .unwrap();

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_span_processor(
            BatchSpanProcessor::builder(span_exporter)
                .with_batch_config(batch_config)
                .build(),
        )
        .with_resource(resource)
        .build();

    let tracer = tracer_provider.tracer("home-gateway");
    global::set_tracer_provider(tracer_provider);

    tracer
}

pub fn init() {
    let tracer = external_tracer();

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    tracing_subscriber::registry()
        .with(
            Targets::default()
                .with_target("otel::tracing", Level::TRACE)
                .with_target("sea_orm::database", Level::TRACE)
                .with_default(Level::INFO),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
}
