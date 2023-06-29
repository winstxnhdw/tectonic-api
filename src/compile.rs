use axum::{
    body::Body,
    http::header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use serde::Deserialize;
use tectonic::latex_to_pdf;

#[derive(Deserialize)]
pub struct CompileSchema {
    latex: String,
}

pub async fn compile(Json(payload): Json<CompileSchema>) -> impl IntoResponse {
    let pdf = latex_to_pdf(payload.latex).expect("Unable to convert LaTeX to PDF");
    let body = Body::from(pdf);

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/pdf")
        .header(CONTENT_DISPOSITION, "attachment; filename=\"compiled.pdf\"")
        .body(body)
        .unwrap()
}
