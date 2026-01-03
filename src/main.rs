use std::env::var;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let max_cache_memory = var("MAX_CACHE_MEMORY")
        .ok()
        .and_then(|key| key.parse().ok())
        .unwrap_or(12884901888);
    let cache_expiry = var("CACHE_EXPIRY")
        .ok()
        .and_then(|key| key.parse().ok())
        .unwrap_or(3600);

    let app = tectonic_api::app(max_cache_memory, std::time::Duration::from_secs(cache_expiry));
    let port = var("SERVER_PORT").unwrap_or("5555".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    axum::serve(listener, app.into_make_service()).await
}
