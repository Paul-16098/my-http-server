use actix_web::{HttpResponse, get, http::header::ContentType, scope, web};
use serde_json::json;
use utoipa::OpenApi;

struct ServerAddon;
impl utoipa::Modify for ServerAddon {
    fn modify(&self, _openapi: &mut utoipa::openapi::OpenApi) {}
}

#[derive(utoipa::OpenApi)]
#[openapi(info(version = crate::VERSION, license(name = "gpl-3.0", url = "/api/license"), contact(name = "GitHub", url = "https://github.com/Paul-16098/my-http-server/")), servers((url = ".", description = "Local server")), modifiers(&ServerAddon), paths(meta, license, file::get_raw_file))]
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
#[scope("/file")]
pub(crate) mod file {
    use std::path::{Path, PathBuf};

    use actix_files::NamedFile;
    use actix_web::{HttpResponse, post};
    use log::warn;

    use crate::{cofg::config::Cofg, error::AppError};

    enum ValidationError {
        Empty,
        Traversal(String),
        NotFound,
        NotFile,
        IoError(std::io::Error),
    }

    impl ValidationError {
        fn into_response(self) -> HttpResponse {
            match self {
                Self::Empty => HttpResponse::BadRequest().body("empty path"),
                Self::Traversal(path) => {
                    warn!("attempt to access file outside public_path: {}", path);
                    HttpResponse::Forbidden().body("path traversal attacks are not allowed")
                }
                Self::NotFound => HttpResponse::NotFound().body("path not exist"),
                Self::NotFile => HttpResponse::BadRequest().body("not a file"),
                Self::IoError(e) => HttpResponse::BadRequest().body(AppError::from(e).to_string()),
            }
        }
    }

    fn validate_and_resolve_path(
        path: &str,
        public_path: &Path,
    ) -> Result<PathBuf, ValidationError> {
        if path.trim().is_empty() {
            return Err(ValidationError::Empty);
        }

        let candidate = public_path.join(path);
        let resolved = candidate.canonicalize().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ValidationError::NotFound
            } else {
                ValidationError::IoError(e)
            }
        })?;

        if !resolved.starts_with(public_path) {
            return Err(ValidationError::Traversal(resolved.display().to_string()));
        }

        if !resolved.is_file() {
            return Err(ValidationError::NotFile);
        }

        Ok(resolved)
    }

    /// Get raw file content
    #[utoipa::path(
        request_body(content = String,description = "file path relative to public_path", example = "./dir/test.md"),
        responses(
            (status = 200, body = String, description = "file content"),
            (status = 403, body = String, description = "path traversal attacks are not allowed"),
            (status = 404, body = String, description = "path not exist"),
            (status = 400, body = String, description = "error reading file"),
        )
    )]
    #[post("/get_raw")]
    async fn get_raw_file(req: actix_web::HttpRequest, path: String) -> HttpResponse {
        let c = Cofg::get(false);
        let public_path = match Path::new(&c.public_path).canonicalize() {
            Ok(v) => v,
            Err(e) => {
                warn!("public_path canonicalize failed: {}", e);
                return HttpResponse::InternalServerError().body(AppError::from(e).to_string());
            }
        };

        let resolved = match validate_and_resolve_path(&path, &public_path) {
            Ok(p) => p,
            Err(e) => return e.into_response(),
        };

        match NamedFile::open_async(&resolved).await {
            Ok(file) => file.into_response(&req),
            Err(e) => HttpResponse::BadRequest().body(AppError::from(e).to_string()),
        }
    }
}
