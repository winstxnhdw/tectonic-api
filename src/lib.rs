mod v1;

use axum::routing::get;
use axum::Router;
use utoipa::OpenApi;

#[utoipauto::utoipauto]
#[derive(OpenApi)]
#[openapi(info(description = "An API for compiling TeX/LaTeX with Tectonic"))]
struct ApiSpecification;

pub fn app() -> Router {
    let root_path = "/api";

    Router::new()
        .nest(
            root_path,
            Router::new().route("/", get(())).merge(v1::router()),
        )
        .merge(
            utoipa_swagger_ui::SwaggerUi::new(format!("{}/docs", root_path))
                .url("/api-docs/openapi.json", ApiSpecification::openapi()),
        )
}
