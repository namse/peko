use color_eyre::eyre::Result;
use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::env;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn setup_otlp() -> Result<(SdkTracerProvider, SdkMeterProvider)> {
    let otlp_endpoint = env::var("OTLP_ENDPOINT").expect("env var OTLP_ENDPOINT is not set");

    let tracer_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&otlp_endpoint)
        .with_protocol(Protocol::Grpc)
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(tracer_exporter)
        .with_resource(Resource::builder().with_service_name("hq").build())
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    let tracer = tracer_provider.tracer("hq-tracer");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer)) // Grafana 전송용
        .init();

    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(&otlp_endpoint)
        .with_protocol(Protocol::Grpc)
        .build()?;

    let reader = PeriodicReader::builder(metric_exporter)
        .with_interval(Duration::from_secs(10))
        .build();

    let meter_provider = SdkMeterProvider::builder()
        .with_resource(Resource::builder().with_service_name("hq").build())
        .with_reader(reader)
        .build();

    global::set_meter_provider(meter_provider.clone());

    info!("otel setup completed");
    Ok((tracer_provider, meter_provider))
}

pub fn on_shutdown(
    (tracer_provider, meter_provider): (SdkTracerProvider, SdkMeterProvider),
) -> Result<()> {
    tracer_provider.shutdown()?;
    meter_provider.shutdown()?;
    Ok(())
}
