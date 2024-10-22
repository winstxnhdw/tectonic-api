use std::{env::var, time::Duration};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let max_cache_memory = var("MAX_CACHE_MEMORY")
        .unwrap_or("12884901888".into())
        .parse()
        .unwrap();
    let cache_expiry = var("CACHE_EXPIRY")
        .unwrap_or("3600".into())
        .parse()
        .unwrap();

    let port = var("SERVER_PORT").unwrap_or("5555".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    let app = tectonic_api::app(max_cache_memory, Duration::from_secs(cache_expiry));

    axum::serve(listener, app.into_make_service()).await
}
