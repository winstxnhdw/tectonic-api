mod consul;

use std::env::var;
use std::fmt;
use std::fmt::Debug;
use std::io::Error as IoError;

enum Error {
    Consul(consul::Error),
    Io(IoError),
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Consul(error) => Debug::fmt(error, formatter),
            Self::Io(error) => Debug::fmt(error, formatter),
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

async fn on_shutdown(consul_registration: Option<consul::Registration>) {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("failed to register SIGTERM handler")
        .recv()
        .await;

    if let Some(registration) = consul_registration {
        registration.deregister().await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let service_name = var("SERVICE_NAME").unwrap_or("tectonic-api".into());
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

    let app = tectonic_api::app(max_cache_memory, std::time::Duration::from_secs(cache_expiry));
    let port = var("SERVER_PORT").unwrap_or("5555".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    let consul_registration = consul::RegistrationBuilder::new()
        .http_addr(consul_http_addr.as_deref())
        .auth_token(consul_auth_token.as_deref())?
        .service_address(consul_service_address.as_deref())
        .service_name(&service_name)
        .service_port(consul_service_port)
        .service_scheme(&consul_service_scheme)
        .register()
        .await?;

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(on_shutdown(consul_registration))
        .await?;

    Ok(())
}
