use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::CONTENT_DISPOSITION;
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct CompileSchema {
    #[param(value_type = String, example = r#"\documentclass{article}\begin{document}Hello, world!\end{document}"#)]
    source: std::sync::Arc<str>,
}

#[utoipa::path(
    get,
    path = "/api/v2/pdf",
    params(CompileSchema),
    responses(
        (status = 200, body = Vec<u8>, content_type = "application/pdf"),
    )
)]
pub async fn pdf(State(state): State<crate::state::AppState>, Query(query): Query<CompileSchema>) -> impl IntoResponse {
    let result = state.cache.try_get_with(query.source.clone(), async {
        crate::features::latex_to_pdf(&query.source).map_err(|e| e.to_string())
    });

    let response = match result.await {
        Ok(bytes) => axum::response::Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/pdf")
            .header(CONTENT_DISPOSITION, r#"inline; filename="index.pdf""#)
            .body(axum::body::Body::from(bytes)),
        Err(e) => axum::response::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header(CONTENT_TYPE, "text/plain")
            .body(e.to_string().into()),
    };

    response.unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, "Unable to build valid response").into_response())
}
