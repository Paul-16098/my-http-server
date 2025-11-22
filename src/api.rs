use actix_web::{HttpResponse, get, http::header::ContentType, scope, web};
use serde_json::json;
use utoipa::OpenApi;

struct ServerAddon;
impl utoipa::Modify for ServerAddon {
    fn modify(&self, _openapi: &mut utoipa::openapi::OpenApi) {}
}

#[derive(utoipa::OpenApi)]
#[openapi(
    info(version = crate::VERSION, license(name = "gpl-3.0", url = "/api/license"), contact(name = "GitHub", url = "https://github.com/Paul-16098/my-http-server/")), 
    servers((url = ".", description = "Local server")), 
    modifiers(&ServerAddon), 
    paths(meta, license, file::get_raw_file, file::file_info, file::list_files, file::check_exists),
    components(schemas(file::FileInfo, file::DirectoryListing, file::ExistsResponse))
)]
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
    use serde::{Deserialize, Serialize};

    use crate::{cofg::config::Cofg, error::AppError};

    enum ValidationError {
        Empty,
        Traversal(String),
        NotFound,
        NotFile,
        NotDirectory,
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
                Self::NotDirectory => HttpResponse::BadRequest().body("not a directory"),
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

    fn validate_and_resolve_any_path(
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

        Ok(resolved)
    }

    fn validate_and_resolve_directory_path(
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

        if !resolved.is_dir() {
            return Err(ValidationError::NotDirectory);
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

    /// Response structure for file metadata
    #[derive(Serialize, Deserialize, utoipa::ToSchema)]
    pub struct FileInfo {
        /// File name
        pub name: String,
        /// Path relative to public_path
        pub path: String,
        /// File size in bytes (None for directories)
        pub size: Option<u64>,
        /// Last modified time (Unix timestamp in seconds)
        pub modified: Option<i64>,
        /// Whether this is a directory
        pub is_directory: bool,
        /// Whether this is a file
        pub is_file: bool,
    }

    /// Response structure for directory listing
    #[derive(Serialize, Deserialize, utoipa::ToSchema)]
    pub struct DirectoryListing {
        /// Directory path relative to public_path
        pub path: String,
        /// List of files and subdirectories
        pub entries: Vec<FileInfo>,
    }

    /// Response structure for path existence check
    #[derive(Serialize, Deserialize, utoipa::ToSchema)]
    pub struct ExistsResponse {
        /// Whether the path exists
        pub exists: bool,
        /// Type of the path if it exists
        pub path_type: Option<String>,
    }

    /// Get file or directory metadata
    #[utoipa::path(
        request_body(content = String, description = "file or directory path relative to public_path", example = "./dir/test.md"),
        responses(
            (status = 200, body = FileInfo, description = "file/directory metadata"),
            (status = 403, body = String, description = "path traversal attacks are not allowed"),
            (status = 404, body = String, description = "path not exist"),
            (status = 400, body = String, description = "error getting metadata"),
        )
    )]
    #[post("/info")]
    async fn file_info(path: String) -> HttpResponse {
        let c = Cofg::get(false);
        let public_path = match Path::new(&c.public_path).canonicalize() {
            Ok(v) => v,
            Err(e) => {
                warn!("public_path canonicalize failed: {}", e);
                return HttpResponse::InternalServerError().body(AppError::from(e).to_string());
            }
        };

        let resolved = match validate_and_resolve_any_path(&path, &public_path) {
            Ok(p) => p,
            Err(e) => return e.into_response(),
        };

        let metadata = match std::fs::metadata(&resolved) {
            Ok(m) => m,
            Err(e) => return HttpResponse::BadRequest().body(AppError::from(e).to_string()),
        };

        let name = resolved
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let relative_path = resolved
            .strip_prefix(&public_path)
            .unwrap_or(&resolved)
            .to_string_lossy()
            .to_string();

        let size = if metadata.is_file() {
            Some(metadata.len())
        } else {
            None
        };

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);

        let info = FileInfo {
            name,
            path: relative_path,
            size,
            modified,
            is_directory: metadata.is_dir(),
            is_file: metadata.is_file(),
        };

        HttpResponse::Ok().json(info)
    }

    /// List files in a directory
    #[utoipa::path(
        request_body(content = String, description = "directory path relative to public_path", example = "./dir"),
        responses(
            (status = 200, body = DirectoryListing, description = "directory contents"),
            (status = 403, body = String, description = "path traversal attacks are not allowed"),
            (status = 404, body = String, description = "path not exist"),
            (status = 400, body = String, description = "not a directory or error reading directory"),
        )
    )]
    #[post("/list")]
    async fn list_files(path: String) -> HttpResponse {
        let c = Cofg::get(false);
        let public_path = match Path::new(&c.public_path).canonicalize() {
            Ok(v) => v,
            Err(e) => {
                warn!("public_path canonicalize failed: {}", e);
                return HttpResponse::InternalServerError().body(AppError::from(e).to_string());
            }
        };

        let resolved = match validate_and_resolve_directory_path(&path, &public_path) {
            Ok(p) => p,
            Err(e) => return e.into_response(),
        };

        let entries_result = std::fs::read_dir(&resolved);
        let entries_iter = match entries_result {
            Ok(e) => e,
            Err(e) => return HttpResponse::BadRequest().body(AppError::from(e).to_string()),
        };

        let mut entries = Vec::new();
        for entry_result in entries_iter {
            let entry = match entry_result {
                Ok(e) => e,
                Err(e) => {
                    warn!("Error reading directory entry: {}", e);
                    continue;
                }
            };

            let entry_path = entry.path();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    warn!("Error getting metadata for {:?}: {}", entry_path, e);
                    continue;
                }
            };

            let name = entry
                .file_name()
                .to_string_lossy()
                .to_string();

            let relative_path = entry_path
                .strip_prefix(&public_path)
                .unwrap_or(&entry_path)
                .to_string_lossy()
                .to_string();

            let size = if metadata.is_file() {
                Some(metadata.len())
            } else {
                None
            };

            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);

            entries.push(FileInfo {
                name,
                path: relative_path,
                size,
                modified,
                is_directory: metadata.is_dir(),
                is_file: metadata.is_file(),
            });
        }

        // Sort entries: directories first, then files, alphabetically within each group
        entries.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        let relative_path = resolved
            .strip_prefix(&public_path)
            .unwrap_or(&resolved)
            .to_string_lossy()
            .to_string();

        let listing = DirectoryListing {
            path: relative_path,
            entries,
        };

        HttpResponse::Ok().json(listing)
    }

    /// Check if a path exists
    #[utoipa::path(
        request_body(content = String, description = "path relative to public_path to check", example = "./dir/test.md"),
        responses(
            (status = 200, body = ExistsResponse, description = "existence check result"),
            (status = 403, body = String, description = "path traversal attacks are not allowed"),
            (status = 400, body = String, description = "invalid path"),
        )
    )]
    #[post("/exists")]
    async fn check_exists(path: String) -> HttpResponse {
        let c = Cofg::get(false);
        let public_path = match Path::new(&c.public_path).canonicalize() {
            Ok(v) => v,
            Err(e) => {
                warn!("public_path canonicalize failed: {}", e);
                return HttpResponse::InternalServerError().body(AppError::from(e).to_string());
            }
        };

        // Check for empty path
        if path.trim().is_empty() {
            return HttpResponse::BadRequest().body("empty path");
        }

        let candidate = public_path.join(&path);
        
        // Try to canonicalize - if it fails with NotFound, the path doesn't exist
        let resolved = match candidate.canonicalize() {
            Ok(p) => p,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return HttpResponse::Ok().json(ExistsResponse {
                    exists: false,
                    path_type: None,
                });
            }
            Err(e) => {
                return HttpResponse::BadRequest().body(AppError::from(e).to_string());
            }
        };

        // Check for path traversal
        if !resolved.starts_with(&public_path) {
            warn!("attempt to access file outside public_path: {}", resolved.display());
            return HttpResponse::Forbidden().body("path traversal attacks are not allowed");
        }

        let path_type = if resolved.is_file() {
            Some("file".to_string())
        } else if resolved.is_dir() {
            Some("directory".to_string())
        } else {
            Some("other".to_string())
        };

        HttpResponse::Ok().json(ExistsResponse {
            exists: true,
            path_type,
        })
    }
}
