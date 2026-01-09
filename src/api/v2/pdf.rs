use axum::extract::Query;
use axum::extract::State;
use axum::http;
use axum::http::header;

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct CompileSchema {
    #[param(example = r#"\documentclass{article}\begin{document}Hello, world!\end{document}"#)]
    source: String,
}

#[utoipa::path(
    get,
    path = "/api/v2/pdf",
    params(CompileSchema),
    responses(
        (status = 200, body = Vec<u8>, content_type = "application/pdf"),
    )
)]
pub async fn pdf(
    State(state): State<crate::state::AppState>,
    Query(query): Query<CompileSchema>,
) -> Result<impl axum::response::IntoResponse, String> {
    let result = state.cache.try_get_with(query.source.clone(), async {
        crate::features::latex_to_pdf(&query.source).map_err(|e| e.to_string())
    });

    http::Response::builder()
        .status(http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_DISPOSITION, "attachment; filename=\"compiled.pdf\"")
        .body(axum::body::Body::from(result.await.map_err(|e| e.to_string())?))
        .map_err(|e| e.to_string())
}
