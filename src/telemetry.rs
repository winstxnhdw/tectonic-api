use axum::Router;
use axum::extract::MatchedPath;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use opentelemetry::KeyValue;
use opentelemetry::global;
use opentelemetry::metrics::Histogram;
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::Protocol;
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::fmt;
use std::fmt::Debug;
use std::sync::OnceLock;
use std::time::Instant;
use tokio::task::JoinHandle;
use tower_http::trace::DefaultMakeSpan;
use tower_http::trace::DefaultOnFailure;
use tower_http::trace::DefaultOnResponse;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::Layer as _;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt as _;

pub enum Error {
    Exporter(opentelemetry_otlp::ExporterBuildError),
    Provider(opentelemetry_sdk::error::OTelSdkError),
    Subscriber(tracing::subscriber::SetGlobalDefaultError),
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exporter(error) => Debug::fmt(error, formatter),
            Self::Provider(error) => Debug::fmt(error, formatter),
            Self::Subscriber(error) => Debug::fmt(error, formatter),
        }
    }
}

impl From<opentelemetry_otlp::ExporterBuildError> for Error {
    fn from(error: opentelemetry_otlp::ExporterBuildError) -> Self {
        Self::Exporter(error)
    }
}

impl From<opentelemetry_sdk::error::OTelSdkError> for Error {
    fn from(error: opentelemetry_sdk::error::OTelSdkError) -> Self {
        Self::Provider(error)
    }
}

impl From<tracing::subscriber::SetGlobalDefaultError> for Error {
    fn from(error: tracing::subscriber::SetGlobalDefaultError) -> Self {
        Self::Subscriber(error)
    }
}

pub struct Telemetry {
    tracer_provider: Option<SdkTracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
    logger_provider: Option<SdkLoggerProvider>,
    process_metrics: JoinHandle<()>,
}

impl Telemetry {
    pub fn init(service_name: &str, service_instance_id: &str) -> Result<Self, Error> {
        let resource = Resource::builder()
            .with_service_name(service_name.to_owned())
            .with_attribute(KeyValue::new("service.instance.id", service_instance_id.to_owned()))
            .build();

        let span_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .build()?;
        let tracer_provider = SdkTracerProvider::builder()
            .with_resource(resource.clone())
            .with_batch_exporter(span_exporter)
            .build();

        let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .build()?;
        let meter_provider = SdkMeterProvider::builder()
            .with_resource(resource.clone())
            .with_periodic_exporter(metric_exporter)
            .build();

        let log_exporter = opentelemetry_otlp::LogExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .build()?;
        let logger_provider = SdkLoggerProvider::builder()
            .with_resource(resource)
            .with_batch_exporter(log_exporter)
            .build();

        global::set_text_map_propagator(TraceContextPropagator::new());
        global::set_tracer_provider(tracer_provider.clone());
        global::set_meter_provider(meter_provider.clone());

        let tracer = tracer_provider.tracer(env!("CARGO_PKG_NAME"));
        let telemetry_filter = Targets::new()
            .with_default(LevelFilter::INFO)
            .with_target("h2", LevelFilter::OFF)
            .with_target("hyper", LevelFilter::OFF)
            .with_target("opentelemetry", LevelFilter::OFF)
            .with_target("opentelemetry_otlp", LevelFilter::OFF)
            .with_target("opentelemetry_sdk", LevelFilter::OFF)
            .with_target("reqwest", LevelFilter::OFF);
        let trace_layer = OpenTelemetryLayer::new(tracer).with_filter(telemetry_filter.clone());
        let log_layer = OpenTelemetryTracingBridge::new(&logger_provider).with_filter(telemetry_filter);
        let subscriber = tracing_subscriber::registry().with(trace_layer).with(log_layer);
        tracing::subscriber::set_global_default(subscriber)?;

        let process_meter = meter_provider.meter("system-metrics");
        let process_metrics = tokio::spawn(async move {
            if let Err(error) = opentelemetry_system_metrics::init_process_observer(process_meter).await {
                tracing::error!(error = ?error, "failed to collect process metrics");
            }
        });

        Ok(Self {
            tracer_provider: Some(tracer_provider),
            meter_provider: Some(meter_provider),
            logger_provider: Some(logger_provider),
            process_metrics,
        })
    }

    pub fn shutdown(mut self) -> Result<(), Error> {
        self.shutdown_providers()
    }

    fn shutdown_providers(&mut self) -> Result<(), Error> {
        self.process_metrics.abort();

        if let Some(provider) = self.tracer_provider.take() {
            provider.shutdown()?;
        }
        if let Some(provider) = self.meter_provider.take() {
            provider.shutdown()?;
        }
        if let Some(provider) = self.logger_provider.take() {
            provider.shutdown()?;
        }

        Ok(())
    }
}

impl Drop for Telemetry {
    fn drop(&mut self) {
        let _ = self.shutdown_providers();
    }
}

pub fn instrument_router(router: Router, enabled: bool) -> Router {
    if !enabled {
        return router;
    }

    router
        .layer(axum::middleware::from_fn(record_http_request_duration))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
}

fn http_request_duration() -> &'static Histogram<f64> {
    static HISTOGRAM: OnceLock<Histogram<f64>> = OnceLock::new();

    HISTOGRAM.get_or_init(|| {
        global::meter(env!("CARGO_PKG_NAME"))
            .f64_histogram("http.server.request.duration")
            .with_description("Duration of HTTP server requests")
            .with_unit("s")
            .build()
    })
}

async fn record_http_request_duration(request: Request, next: Next) -> Response {
    let started_at = Instant::now();
    let method = request.method().as_str().to_owned();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|path| path.as_str().to_owned());

    let response = next.run(request).await;

    let mut attributes = vec![
        KeyValue::new("http.request.method", method),
        KeyValue::new("http.response.status_code", i64::from(response.status().as_u16())),
    ];

    if let Some(route) = route {
        attributes.push(KeyValue::new("http.route", route));
    }

    http_request_duration().record(started_at.elapsed().as_secs_f64(), &attributes);

    response
}
