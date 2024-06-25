#[utoipa::path(get, path = "/v1", responses((status = 200, body = String)))]
pub async fn index() -> &'static str {
    "Welcome to v1 of the API!"
}
