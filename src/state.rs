use gxhash::GxBuildHasher;

pub struct AppState {
    pub cache: moka::future::Cache<String, Vec<u8>, GxBuildHasher>,
}
