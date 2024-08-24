mod v1;

use axum::routing::get;
use axum::Router;
use utoipa::OpenApi;

#[utoipauto::utoipauto]
#[derive(OpenApi)]
#[openapi(info(description = "An API for compiling TeX/LaTeX with Tectonic"))]
struct ApiSpecification;

pub fn app() -> Router {
    Router::new().route("/", get(())).merge(v1::router()).merge(
        utoipa_swagger_ui::SwaggerUi::new("/docs")
            .url("/api-docs/openapi.json", ApiSpecification::openapi()),
    )
}
