use axum::Json;
use axum::extract::State;
use axum::http;
use axum::http::header;

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
    State(state): State<crate::state::AppState>,
    Json(payload): Json<CompileSchema>,
) -> Result<impl axum::response::IntoResponse, String> {
    let result = state.cache.try_get_with(payload.latex.clone(), async {
        crate::features::latex_to_pdf(&payload.latex).map_err(|e| e.to_string())
    });

    http::Response::builder()
        .status(http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_DISPOSITION, "attachment; filename=\"compiled.pdf\"")
        .body(axum::body::Body::from(result.await.map_err(|e| e.to_string())?))
        .map_err(|e| e.to_string())
}
