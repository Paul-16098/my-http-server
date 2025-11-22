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
    components(schemas(file::FileInfo, file::DirectoryListing, file::ExistsResponse, file::PathType))
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

    /// Helper to canonicalize the configured public_path with unified error handling.
    /// WHY: All endpoints need a canonical, absolute root path to enforce prefix checks
    /// for traversal protection. Centralizing this logic removes duplication and ensures
    /// future security fixes apply everywhere.
    fn get_canonical_public_path() -> Result<PathBuf, HttpResponse> {
        let c = Cofg::get(false);
        Path::new(&c.public_path).canonicalize().map_err(|e| {
            warn!("public_path canonicalize failed: {}", e);
            HttpResponse::InternalServerError().body(AppError::from(e).to_string())
        })
    }

    /// Common validation logic for all path types
    fn validate_path_base(path: &str, public_path: &Path) -> Result<PathBuf, ValidationError> {
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

    fn validate_and_resolve_path(
        path: &str,
        public_path: &Path,
    ) -> Result<PathBuf, ValidationError> {
        let resolved = validate_path_base(path, public_path)?;

        if !resolved.is_file() {
            return Err(ValidationError::NotFile);
        }

        Ok(resolved)
    }

    fn validate_and_resolve_any_path(
        path: &str,
        public_path: &Path,
    ) -> Result<PathBuf, ValidationError> {
        validate_path_base(path, public_path)
    }

    fn validate_and_resolve_directory_path(
        path: &str,
        public_path: &Path,
    ) -> Result<PathBuf, ValidationError> {
        let resolved = validate_path_base(path, public_path)?;

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
        let public_path = match get_canonical_public_path() {
            Ok(v) => v,
            Err(resp) => return resp,
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
    #[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema)]
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
    #[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema)]
    pub struct DirectoryListing {
        /// Directory path relative to public_path
        pub path: String,
        /// List of files and subdirectories
        pub entries: Vec<FileInfo>,
    }

    /// Type of the path if it exists: "file", "directory", or "other"
    #[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema, PartialEq, Eq)]
    #[serde(rename_all = "lowercase")]
    pub enum PathType {
        File,
        Directory,
        Other,
    }

    /// Response structure for path existence check
    #[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema)]
    pub struct ExistsResponse {
        /// Whether the path exists
        pub exists: bool,
        /// Type of the path if it exists: "file", "directory", or "other"
        #[schema(example = "file")]
        pub path_type: Option<PathType>,
    }

    /// Get file or directory metadata
    ///
    /// WHY: Allows API consumers to query metadata without downloading full file content.
    /// Critical for building file browsers and checking file properties.
    ///
    /// # Security
    /// - Validates against path traversal via canonicalization + prefix check
    /// - Only exposes files within configured public_path
    ///
    /// # Performance
    /// Synchronous filesystem access; may block on slow filesystems
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
        let public_path = match get_canonical_public_path() {
            Ok(v) => v,
            Err(resp) => return resp,
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
            .unwrap_or_else(|e| {warn!("{e}");&resolved})
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
    ///
    /// WHY: Enables building file browsers and navigation UIs that need to display
    /// directory contents. Sorts deterministically (directories first, then alphabetically)
    /// for consistent client-side rendering.
    ///
    /// # Security
    /// - Validates directory path against traversal attacks
    /// - Skips entries with metadata errors (logs warnings) rather than failing entire request
    ///
    /// # Performance
    /// Synchronous directory traversal; may be slow for large directories
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
        let public_path = match get_canonical_public_path() {
            Ok(v) => v,
            Err(resp) => return resp,
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

            let relative_path = match entry_path.strip_prefix(&public_path) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(e) => {
                    warn!("Failed to strip prefix from directory entry: {}: {}", entry_path.display(), e);
                    continue; // Skip inconsistent entry, avoid exposing error text to API consumers
                }
            };

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
        // NOTE: Use !is_directory so directories (true) sort before files (false)
        entries.sort_by_cached_key(|e| (!e.is_directory, e.name.to_lowercase()));

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
    ///
    /// WHY: Provides a lightweight existence probe without incurring the overhead of
    /// metadata extraction or content streaming. Designed to return `200 OK` with
    /// `exists:false` for missing paths (instead of a 404) so client code can perform
    /// conditional logic (e.g. create-if-missing) without treating absence as an error.
    /// This keeps error semantics reserved for invalid requests (empty path, traversal, IO
    /// errors) and simplifies consumer retry logic.
    ///
    /// Rationale: Reuses centralized validation helper ensuring any future hardening of
    /// traversal or normalization automatically applies here. Separating existence checks
    /// from `/info` avoids eager filesystem metadata calls for hot-path probes.
    ///
    /// # Security
    /// - Centralized path validation prevents traversal attempts
    /// - Missing paths never leak internal error details
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
        let public_path = match get_canonical_public_path() {
            Ok(v) => v,
            Err(resp) => return resp,
        };

        let resolved = match validate_and_resolve_any_path(&path, &public_path) {
            Ok(p) => p,
            Err(ValidationError::NotFound) => {
                return HttpResponse::Ok().json(ExistsResponse {
                    exists: false,
                    path_type: None,
                });
            }
            Err(e) => return e.into_response(),
        };

        let path_type = if resolved.is_file() {
            Some(PathType::File)
        } else if resolved.is_dir() {
            Some(PathType::Directory)
        } else {
            Some(PathType::Other)
        };

        HttpResponse::Ok().json(ExistsResponse {
            exists: true,
            path_type,
        })
    }
}
