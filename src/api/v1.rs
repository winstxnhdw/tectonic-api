pub mod compile;

use axum::Router;

pub fn router(shared_state: crate::state::AppState) -> Router {
    Router::new()
        .route("/v1/compile", axum::routing::post(compile::compile))
        .with_state(shared_state)
}
