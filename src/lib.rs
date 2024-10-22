mod state;
mod v1;

use axum::Router;
use utoipa::OpenApi;

#[utoipauto::utoipauto]
#[derive(OpenApi)]
#[openapi(info(description = "An API for compiling TeX/LaTeX with Tectonic"))]
struct ApiSpecification;

pub fn app() -> Router {
    let root_path = "/api";
    let shared_state = std::sync::Arc::new(state::AppState {
        cache: moka::future::Cache::builder()
            .weigher(|_, v: &Vec<u8>| v.len().try_into().unwrap_or(u32::MAX))
            .max_capacity(12 * 1024 * 1024 * 1024)
            .time_to_idle(std::time::Duration::from_secs(3600))
            .build_with_hasher(gxhash::GxBuildHasher::default()),
    });

    Router::new()
        .nest(
            root_path,
            Router::new()
                .route("/", axum::routing::get(()))
                .merge(v1::router(shared_state)),
        )
        .merge(
            utoipa_swagger_ui::SwaggerUi::new(format!("{}/docs", root_path))
                .url("/api-docs/openapi.json", ApiSpecification::openapi()),
        )
}
