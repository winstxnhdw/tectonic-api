mod v1;

use axum::Router;
use utoipa::OpenApi;

#[utoipauto::utoipauto]
#[derive(OpenApi)]
#[openapi(info(description = "An API for compiling TeX/LaTeX with Tectonic"))]
struct ApiSpecification;

fn on_startup() {
    _ = tectonic::latex_to_pdf(
        r#"\documentclass{article}\begin{document}Hello, world!\end{document}"#,
    );
}

pub fn app() -> Router {
    on_startup();

    Router::new().merge(v1::router()).merge(
        utoipa_swagger_ui::SwaggerUi::new("/docs")
            .url("/api-docs/openapi.json", ApiSpecification::openapi()),
    )
}
