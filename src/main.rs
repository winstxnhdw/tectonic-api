use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router, Server,
};
use serde::Deserialize;
use std::env::var;
use tectonic::latex_to_pdf;
use tracing_subscriber::fmt;

#[derive(Deserialize)]
struct CompileSchema {
    latex: String,
}

async fn root() -> &'static str {
    "Welcome to v1 of the API!"
}

async fn compile(Json(payload): Json<CompileSchema>) -> impl IntoResponse {
    let pdf = latex_to_pdf(payload.latex).expect("Unable to convert LaTeX to PDF");
    let body = axum::body::Body::from(pdf);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"compiled.pdf\"",
        )
        .body(body)
        .unwrap()
}

#[tokio::main]
async fn main() {
    fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/compile", post(compile));

    let port = var("SERVER_PORT").unwrap();

    Server::bind(&format!("0.0.0.0:{port}").parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
