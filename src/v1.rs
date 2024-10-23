pub mod compile;
pub mod index;

use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;

pub fn router(shared_state: AppState) -> Router {
    Router::new()
        .route("/v1", get(index::index))
        .route("/v1/compile", post(compile::compile))
        .with_state(shared_state)
}
