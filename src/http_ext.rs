//! HttpRequest per-request cached helpers
//!
//! WHY: Avoid recomputing derived request values (decoded URI, resolved disk path, extension
//! checks). Actix's `Extensions` offers cheap per-request storage; this reduces repeated percent
//! decode, path joins, & extension parsing when multiple middleware / handlers need them.
//!
//! 中文：使用每請求快取避免重複 percent-decode 與路徑組合，降低日誌或處理流程的重工。
use actix_web::{ HttpMessage, HttpRequest };
use std::path::{ Path, PathBuf };

use crate::Cofg;

// Newtype keys for extensions cache
#[derive(Debug)]
struct FilenamePath(PathBuf);
#[derive(Debug)]
struct PublicReqPath(PathBuf);
#[derive(Debug)]
struct IsMarkdown(bool);

/// Cached helpers for HttpRequest
pub trait HttpRequestCachedExt {
  /// Router-captured filename path (from match_info "filename"). Path segmentation deferred to
  /// filesystem operations; caching avoids repeating `match_info` lookup.
  fn cached_filename_path(&self) -> PathBuf;

  /// Absolute path on disk under `public_path` joined with filename.
  /// WHY: Compose once; used for existence check, extension classification, and file reading.
  fn cached_public_req_path(&self, c: &Cofg) -> PathBuf;

  /// Whether the requested file has extension `.md`.
  /// WHY: Determines dynamic render vs static file path.
  fn cached_is_markdown(&self, c: &Cofg) -> bool;
}

impl HttpRequestCachedExt for HttpRequest {
  fn cached_filename_path(&self) -> PathBuf {
    if let Some(v) = self.extensions().get::<FilenamePath>() {
      return v.0.clone();
    }
    let filename_str = self.match_info().query("filename");
    // PathBuf parsing is infallible for plain strings
    let path = PathBuf::from(filename_str);
    self.extensions_mut().insert(FilenamePath(path.clone()));
    path
  }

  fn cached_public_req_path(&self, c: &Cofg) -> PathBuf {
    if let Some(v) = self.extensions().get::<PublicReqPath>() {
      return v.0.clone();
    }
    let path = Path::new(&c.public_path).join(self.cached_filename_path());
    self.extensions_mut().insert(PublicReqPath(path.clone()));
    path
  }

  fn cached_is_markdown(&self, c: &Cofg) -> bool {
    if let Some(v) = self.extensions().get::<IsMarkdown>() {
      return v.0;
    }
    let is_md =
      self
        .cached_public_req_path(c)
        .extension()
        .and_then(|v| v.to_str()) == Some("md");
    self.extensions_mut().insert(IsMarkdown(is_md));
    is_md
  }
}
