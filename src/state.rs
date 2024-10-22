use gxhash::GxBuildHasher;

pub struct AppState {
    pub cache: moka::future::Cache<std::string::String, Vec<u8>, GxBuildHasher>,
}
