mod api;
mod features;
mod state;

use axum::Router;
use utoipa::OpenApi;
use utoipa_scalar::Servable;

#[utoipauto::utoipauto]
#[derive(OpenApi)]
#[openapi(info(description = "An API for compiling TeX/LaTeX with Tectonic"))]
struct ApiSpecification;

pub fn app(max_cache_memory: u64, cache_expiry: std::time::Duration) -> Router {
    let root_path = "/api";
    let cache = moka::future::Cache::builder()
        .weigher(|_, v: &Vec<u8>| v.len().try_into().unwrap_or(u32::MAX))
        .max_capacity(max_cache_memory)
        .time_to_idle(cache_expiry)
        .build_with_hasher(gxhash::GxBuildHasher::default());

    let scalar = utoipa_scalar::Scalar::with_url("/schema/scalar", ApiSpecification::openapi());
    let v2 = Router::new()
        .route("/health", axum::routing::get(api::health::health))
        .merge(api::v2::router(state::AppState { cache }));

    Router::new().nest(root_path, v2).merge(scalar)
}
