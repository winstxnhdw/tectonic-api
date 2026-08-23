mod consul;
mod telemetry;

use std::env::var;
use std::fmt;
use std::fmt::Debug;
use std::io::Error as IoError;

enum Error {
    Consul(consul::Error),
    Io(IoError),
    Telemetry(telemetry::Error),
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Consul(error) => Debug::fmt(error, formatter),
            Self::Io(error) => Debug::fmt(error, formatter),
            Self::Telemetry(error) => Debug::fmt(error, formatter),
        }
    }
}

impl From<consul::Error> for Error {
    fn from(error: consul::Error) -> Self {
        Self::Consul(error)
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}

impl From<telemetry::Error> for Error {
    fn from(error: telemetry::Error) -> Self {
        Self::Telemetry(error)
    }
}

struct Shutdown {
    consul_registration: Option<consul::Registration>,
    telemetry: Option<telemetry::Telemetry>,
}

async fn on_shutdown(shutdown: Shutdown) {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("failed to register SIGTERM handler")
        .recv()
        .await;

    if let Some(registration) = shutdown.consul_registration {
        registration.deregister().await;
    }

    if let Some(telemetry) = shutdown.telemetry
        && let Err(error) = telemetry.shutdown()
    {
        tracing::error!(?error, "failed to shut down telemetry");
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let service_name = var("SERVICE_NAME").unwrap_or("tectonic-api".into());
    let service_instance_id = format!("{service_name}-{:04x}", uuid::Uuid::new_v4().as_u128() as u16);
    let port = var("SERVER_PORT").unwrap_or("5555".into());
    let max_cache_memory = var("MAX_CACHE_MEMORY")
        .ok()
        .and_then(|key| key.parse().ok())
        .unwrap_or(12884901888);
    let cache_expiry = var("CACHE_EXPIRY")
        .ok()
        .and_then(|key| key.parse().ok())
        .unwrap_or(3600);
    let consul_service_port = var("CONSUL_SERVICE_PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(443);
    let consul_http_addr = var("CONSUL_HTTP_ADDR").ok();
    let consul_auth_token = var("CONSUL_AUTH_TOKEN").ok();
    let consul_service_address = var("CONSUL_SERVICE_ADDRESS").ok();
    let consul_service_scheme = var("CONSUL_SERVICE_SCHEME").unwrap_or("https".into());
    let telemetry = var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .ok()
        .filter(|endpoint| !endpoint.trim().is_empty())
        .map(|_| telemetry::Telemetry::init(&service_name, &service_instance_id))
        .transpose()?;

    let consul_registration = consul::RegistrationBuilder::new()
        .http_addr(consul_http_addr.as_deref())
        .auth_token(consul_auth_token.as_deref())?
        .service_address(consul_service_address.as_deref())
        .service_name(&service_name)
        .service_id(service_instance_id)
        .service_port(consul_service_port)
        .service_scheme(&consul_service_scheme)
        .register()
        .await?;

    let app = telemetry::instrument_router(
        tectonic_api::app(max_cache_memory, std::time::Duration::from_secs(cache_expiry)),
        telemetry.is_some(),
    );

    let serve = axum::serve(
        tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?,
        app.into_make_service(),
    );

    let shutdown_dependencies = Shutdown {
        consul_registration,
        telemetry,
    };

    serve.with_graceful_shutdown(on_shutdown(shutdown_dependencies)).await?;

    Ok(())
}
