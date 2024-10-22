use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::state::AppState;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CompileSchema {
    #[schema(example = "\\documentclass{article}\\begin{document}Hello, world!\\end{document}")]
    latex: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/compile",
    request_body = CompileSchema,
    responses(
        (status = 200, body = Vec<u8>, content_type = "application/pdf"),
        (status = 400, body = String)
    )
)]
pub async fn compile(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CompileSchema>,
) -> impl IntoResponse {
    let result = state
        .cache
        .try_get_with(payload.latex.clone(), async {
            match tectonic::latex_to_pdf(payload.latex) {
                Ok(pdf) => Ok(pdf),
                Err(error) => Err(error.to_string()),
            }
        })
        .await;

    match result {
        Ok(pdf) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/pdf")
            .header(CONTENT_DISPOSITION, "attachment; filename=\"compiled.pdf\"")
            .body(Body::from(pdf))
            .unwrap(),

        Err(error) => {
            eprintln!("{:?}", error);

            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(CONTENT_TYPE, "text/plain")
                .body("Failed to compile to PDF!".into())
                .unwrap()
        }
    }
}
