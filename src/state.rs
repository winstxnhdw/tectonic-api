#[derive(Clone)]
pub struct AppState {
    pub cache: moka::future::Cache<std::sync::Arc<str>, Vec<u8>, gxhash::GxBuildHasher>,
}
