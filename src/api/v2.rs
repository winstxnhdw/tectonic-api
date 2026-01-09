pub mod pdf;

use axum::Router;

pub fn router(shared_state: crate::state::AppState) -> Router {
    Router::new()
        .route("/v2/pdf", axum::routing::get(pdf::pdf))
        .with_state(shared_state)
}
