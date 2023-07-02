use axum::body::Body;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use tectonic::latex_to_pdf;
use tracing::error;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CompileSchema {
    #[schema(example = "\\documentclass{article}\\begin{document}Hello, world!\\end{document}")]
    latex: String,
}

#[utoipa::path(
    post,
    path = "/v1/compile",
    request_body = CompileSchema,
    responses(
        (status = 200, body = Vec<u8>, content_type = "application/pdf"),
        (status = 500, body = String, content_type = "text/plain")
    )
)]
pub async fn compile(Json(payload): Json<CompileSchema>) -> impl IntoResponse {
    match latex_to_pdf(payload.latex) {
        Ok(pdf) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/pdf")
            .header(CONTENT_DISPOSITION, "attachment; filename=\"compiled.pdf\"")
            .body(Body::from(pdf))
            .unwrap(),
        Err(error) => {
            error!("Failed to compile: {error}");

            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(CONTENT_TYPE, "text/plain")
                .body(Body::from("Failed to compile to PDF!"))
                .unwrap()
        }
    }
}
