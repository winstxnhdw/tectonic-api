pub mod compile;
pub mod index;

use axum::{
    routing::{get, post},
    Router,
};

pub fn router() -> Router {
    Router::new()
        .route("/v1", get(index::index))
        .route("/v1/compile", post(compile::compile))
}
