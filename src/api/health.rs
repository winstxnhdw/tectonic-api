#[derive(serde::Serialize, utoipa::ToSchema)]

pub struct HealthSchema {
    #[serde(rename = "schemaVersion")]
    schema_version: i32,
    label: &'static str,
    message: &'static str,
}

#[utoipa::path(get, path = "/health", responses((status = 200, body = HealthSchema)))]
pub async fn health() -> impl axum::response::IntoResponse {
    let health = HealthSchema {
        schema_version: 1,
        label: "tectonic-api",
        message: "online",
    };

    axum::Json(health)
}
