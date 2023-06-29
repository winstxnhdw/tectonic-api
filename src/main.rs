mod compile;
mod index;

use axum::{
    routing::{get, post},
    Router, Server,
};

use compile::compile;
use index::index;
use std::env::var;
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() {
    fmt::init();

    let app = Router::new()
        .route("/", get(index))
        .route("/compile", post(compile));

    let port = var("SERVER_PORT").unwrap();

    Server::bind(&format!("0.0.0.0:{port}").parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
