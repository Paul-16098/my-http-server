//! Unified error types using thiserror
//!
//! WHY: Collapse disparate library & IO errors into a small enum convertible to an Actix
//! Responder—simplifies bubbling errors up from deep helpers (markdown, glob, template) without
//! sprinkling HTTP response shaping logic throughout.
//!
//! 中文：集中管理錯誤型別並直接實作 Responder，讓內層只需 `?` 傳遞，不需關心 HTTP 回應細節。

use log::warn;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AppError {
  #[error("IO error: {0}")] Io(#[from] std::io::Error),
  #[error("Glob pattern error: {0}")] GlobPattern(#[from] wax::BuildError),
  #[error("Glob walk error: {0}")] GlobWalk(#[from] wax::WalkError),
  #[error("Template error: {0}")] Template(#[from] mystical_runic::RuneError),
  #[error("Markdown parse error: {0}")] MarkdownParse(String),
  #[error("Config error: {0}")] Config(#[from] config::ConfigError),
  #[error("StripPrefixError: {0}")] StripPrefixError(#[from] std::path::StripPrefixError),
  #[error("Other error: {0}")] Other(String),
}

impl actix_web::Responder for AppError {
  type Body = actix_web::body::BoxBody;

  fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
    warn!("{self}");
    actix_web::HttpResponseBuilder
      ::new(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
      .body(self.to_string())
  }
}

impl From<nom::Err<nom::error::Error<&str>>> for AppError {
  fn from(e: nom::Err<nom::error::Error<&str>>) -> Self {
    AppError::MarkdownParse(e.to_string())
  }
}

pub(crate) type AppResult<T> = Result<T, AppError>;
