pub mod compile;
pub mod health;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

pub fn router(shared_state: AppState) -> Router {
    Router::new()
        .route("/v1", get(health::health))
        .route("/v1/compile", post(compile::compile))
        .with_state(shared_state)
}
