//! Unified error types using thiserror

use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AppError {
  #[error("IO error: {0}")] Io(#[from] std::io::Error),
  #[error("Glob pattern error: {0}")] GlobPattern(#[from] wax::BuildError),
  #[error("Glob walk error: {0}")] GlobWalk(#[from] wax::WalkError),
  #[error("Template error: {0}")] Template(#[from] mystical_runic::RuneError),
  #[error("Notify error: {0}")] Notify(#[from] notify::Error),
  #[error("Markdown parse error: {0}")] MarkdownParse(String),
  #[error("Config error: {0}")] Config(#[from] config::ConfigError),
  #[error("Other: {0}")] Other(String),
}

impl From<nom::Err<nom::error::Error<&str>>> for AppError {
  fn from(e: nom::Err<nom::error::Error<&str>>) -> Self {
    AppError::MarkdownParse(e.to_string())
  }
}

pub(crate) type AppResult<T> = Result<T, AppError>;
