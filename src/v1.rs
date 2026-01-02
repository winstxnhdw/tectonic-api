pub mod compile;
pub mod index;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

pub fn router(shared_state: AppState) -> Router {
    Router::new()
        .route("/v1", get(index::index))
        .route("/v1/compile", post(compile::compile))
        .with_state(shared_state)
}
