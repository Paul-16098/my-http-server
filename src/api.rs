use actix_web::{HttpResponse, get, http::header::ContentType, web};
use serde_json::json;
use utoipa::OpenApi;

struct ServerAddon;
impl utoipa::Modify for ServerAddon {
    fn modify(&self, _openapi: &mut utoipa::openapi::OpenApi) {}
}

#[derive(utoipa::OpenApi)]
#[openapi(info(version = crate::VERSION, license(name = "gpl-3.0", url = "/api/license"), contact(name = "GitHub", url = "https://github.com/Paul-16098/my-http-server/")), servers((url = ".", description = "Local server")), modifiers(&ServerAddon), paths(meta, license))]
pub(crate) struct ApiDoc;

/// Serve the Swagger UI HTML interface for API documentation
///
/// # Returns
/// An HTML page providing interactive API documentation via Swagger UI.
#[get("")]
async fn docs() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("./swagger-ui.html"))
}

#[get("/raw")]
async fn raw_openapi() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ApiDoc::openapi().to_pretty_json().unwrap())
}
/// Get server meta information
/// # Returns
/// A JSON object containing the server version
#[utoipa::path(
    responses(
        (status = 200, body = serde_json::Value)
    )
)]
#[get("/meta")]
async fn meta() -> actix_web::web::Json<serde_json::Value> {
    web::Json(json!({ "VERSION": crate::VERSION,}))
}
/// Get server license
/// # Returns
/// The full text of the server's license
#[utoipa::path(
    responses(
        (status = 200, body = String)
    )
)]
#[get("/license")]
async fn license() -> &'static str {
    include_str!("../LICENSE.txt")
}
