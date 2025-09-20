//! HttpRequest per-request cached helpers
use actix_web::{ HttpRequest, HttpMessage };
use percent_encoding::percent_decode;
use std::borrow::Cow;
use std::path::{ Path, PathBuf };

use crate::Cofg;

// Newtype keys for extensions cache
#[derive(Debug)]
struct DecodedUri(String);
#[derive(Debug)]
struct FilenamePath(PathBuf);
#[derive(Debug)]
struct PublicReqPath(PathBuf);
#[derive(Debug)]
struct IsMarkdown(bool);

/// Cached helpers for HttpRequest
pub trait HttpRequestCachedExt {
  /// Percent-decoded request URI as String (leading '/' trimmed to align with logger format)
  fn cached_decoded_uri(&self) -> String;

  /// Router-captured filename path (from match_info "filename"), parsed as PathBuf
  fn cached_filename_path(&self) -> PathBuf;

  /// Absolute path on disk under public_path joined with filename
  fn cached_public_req_path(&self, c: &Cofg) -> PathBuf;

  /// Whether the requested file has extension .md
  fn cached_is_markdown(&self, c: &Cofg) -> bool;
}

impl HttpRequestCachedExt for HttpRequest {
  fn cached_decoded_uri(&self) -> String {
    if let Some(v) = self.extensions().get::<DecodedUri>() {
      return v.0.clone();
    }
    let u = self.uri().to_string();
    let mut u = percent_decode(u.as_bytes()).decode_utf8().unwrap_or(Cow::Borrowed(&u)).to_string();
    if u.starts_with('/') {
      u.remove(0);
    }
    self.extensions_mut().insert(DecodedUri(u.clone()));
    u
  }

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
