mod compile;
mod index;

use axum::routing::{get, post};
use axum::{serve, Router};
use compile::{compile, CompileSchema};
use index::index;
use std::env::var;
use tracing_subscriber::fmt;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(index::index, compile::compile),
    components(schemas(CompileSchema)),
    info(description = "An API for compiling TeX/LaTeX with Tectonic")
)]
struct ApiSpecification;

#[tokio::main]
async fn main() {
    fmt::init();

    let port = var("SERVER_PORT").unwrap();
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    let app = Router::new()
        .route("/v1", get(index))
        .route("/v1/compile", post(compile))
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiSpecification::openapi()));

    serve(listener, app).await.unwrap();
}
