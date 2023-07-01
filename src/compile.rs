use axum::body::Body;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use retry::delay::NoDelay;
use retry::retry;
use serde::Deserialize;
use tectonic::latex_to_pdf;
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
        (status = 500)
    )
)]
pub async fn compile(Json(payload): Json<CompileSchema>) -> impl IntoResponse {
    let pdf = retry(NoDelay.take(1), || latex_to_pdf(&payload.latex))
        .expect("Unable to convert LaTeX to PDF");
    let body = Body::from(pdf);

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/pdf")
        .header(CONTENT_DISPOSITION, "attachment; filename=\"compiled.pdf\"")
        .body(body)
        .unwrap()
}
