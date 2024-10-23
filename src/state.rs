use gxhash::GxBuildHasher;

#[derive(Clone)]
pub struct AppState {
    pub cache: moka::future::Cache<String, Vec<u8>, GxBuildHasher>,
}
