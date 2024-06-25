#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| String::from("5555"));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    axum::serve(listener, tectonic_api::app().into_make_service()).await
}
