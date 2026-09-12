//! Tracing subscriber. JSON in prod (`RWK_LOG_FORMAT=json`), pretty otherwise.
//! Optional OTLP export behind the `otel` feature.

use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::{Config, LogFormat};

/// Install the global tracing subscriber. Call once at process start.
pub fn init(config: &Config) -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    #[cfg(feature = "otel")]
    {
        let registry = tracing_subscriber::registry()
            .with(filter)
            .with(otel_layer()?);
        match config.log_format {
            LogFormat::Json => registry
                .with(tracing_subscriber::fmt::layer().json())
                .init(),
            LogFormat::Pretty => registry
                .with(tracing_subscriber::fmt::layer().pretty())
                .init(),
        }
    }

    #[cfg(not(feature = "otel"))]
    {
        let registry = tracing_subscriber::registry().with(filter);
        match config.log_format {
            LogFormat::Json => registry
                .with(tracing_subscriber::fmt::layer().json())
                .init(),
            LogFormat::Pretty => registry
                .with(tracing_subscriber::fmt::layer().pretty())
                .init(),
        }
    }

    Ok(())
}

#[cfg(feature = "otel")]
fn otel_layer<S>()
-> anyhow::Result<tracing_opentelemetry::OpenTelemetryLayer<S, opentelemetry_sdk::trace::Tracer>>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::SpanExporter;
    use opentelemetry_sdk::Resource;
    use opentelemetry_sdk::trace::SdkTracerProvider;

    let exporter = SpanExporter::builder().with_http().build()?;
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::builder().with_service_name("rwk-api").build())
        .build();
    opentelemetry::global::set_tracer_provider(provider.clone());
    let tracer = provider.tracer("rwk-api");
    // Provider must outlive the process so the batch exporter can flush.
    std::mem::forget(provider);
    Ok(tracing_opentelemetry::layer().with_tracer(tracer))
}
