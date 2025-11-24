//! Unified error types using thiserror
//!
//! WHY: Collapse disparate library & IO errors into a small enum convertible to an Actix
//! Responder—simplifies bubbling errors up from deep helpers (markdown, glob, template) without
//! sprinkling HTTP response shaping logic throughout.
//!
//! 中文：集中管理錯誤型別並直接實作 Responder，讓內層只需 `?` 傳遞，不需關心 HTTP 回應細節。

use actix_web::{HttpResponse, http::StatusCode};
use log::warn;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Glob pattern error: {0}")]
    GlobPatternError(#[from] wax::BuildError),
    #[error("Glob walk error: {0}")]
    GlobWalkError(#[from] wax::WalkError),
    #[error("Template error: {0}")]
    TemplateError(#[from] handlebars::TemplateError),
    #[error("Render error: {0}")]
    RenderError(#[from] handlebars::RenderError),
    #[error("Markdown parse error: {0}")]
    MarkdownParseError(String),
    #[error("Config error: {0}")]
    ConfigError(#[from] config::ConfigError),
    #[error("StripPrefixError: {0}")]
    StripPrefixError(#[from] std::path::StripPrefixError),
    #[error("TLS Error: {0}")]
    TLSError(#[from] rustls_pki_types::pem::Error),
    #[error("cli Error: {0}")]
    CliError(String),
    #[error("Other error: {0}")]
    OtherError(String),
}

impl actix_web::Responder for AppError {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        // Delegate to ResponseError for consistent status mapping and body shaping
        actix_web::ResponseError::error_response(&self).map_into_boxed_body()
    }
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Io(e) => match e.kind() {
                std::io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
                std::io::ErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
                std::io::ErrorKind::InvalidInput => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            AppError::GlobPatternError(_)
            | AppError::GlobWalkError(_)
            | AppError::TemplateError(_)
            | AppError::RenderError(_)
            | AppError::MarkdownParseError(_)
            | AppError::ConfigError(_)
            | AppError::StripPrefixError(_)
            | AppError::TLSError(_)
            | AppError::CliError(_)
            | AppError::OtherError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        // Log the detailed error server-side, but avoid leaking internals to clients
        warn!("{self}");

        HttpResponse::build(status)
            .content_type(actix_web::http::header::ContentType::plaintext())
            .body(self.to_string())
    }
}

impl From<nom::Err<nom::error::Error<&str>>> for AppError {
    fn from(e: nom::Err<nom::error::Error<&str>>) -> Self {
        AppError::MarkdownParseError(e.to_string())
    }
}

pub(crate) type AppResult<T> = Result<T, AppError>;
