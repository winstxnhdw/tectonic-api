#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let port = std::env::var("SERVER_PORT").unwrap_or("5555".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    axum::serve(listener, tectonic_api::app().into_make_service()).await
}
