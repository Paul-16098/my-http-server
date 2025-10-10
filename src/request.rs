//! HTTP request handlers (routing & rendering)
//!
//! This module wires the two public routes and centralizes how files are resolved and
//! Markdown is rendered into the site template shell.
//!
//! What it does
//! - GET "/" (index):
//!   - If `public/index.html` exists, return it as-is.
//!   - Otherwise, build a dynamic table-of-contents (TOC) via `parser::markdown::get_toc`
//!     and render it with `md2html` into the HTML template.
//! - GET "/{filename:.*}" (fallback):
//!   - Resolve the requested path under `Cofg.public_path`.
//!   - If missing: try `meta/404.html`; else respond with plain-text 404.
//!   - If it is a Markdown file: `read_to_string` + `md2html` with context `path:<resolved>`.
//!   - Otherwise: stream as a static file (`actix_files::NamedFile`).
//!
//! Why it’s designed this way
//! - Keep the index behavior special-cased (let users override with their own `index.html`),
//!   while the fallback route uniformly handles “render Markdown or serve static”.
//! - The request-level helpers in `http_ext` cache decoded URI, computed public path, and
//!   “is markdown” checks to avoid repeated work in hot paths.
//!
//! Templating context rules
//! - Each Markdown render injects per-page `path` into the template context (plus engine
//!   built-ins like `server-version`). The actual HTML fragment from Markdown is placed
//!   into the template’s body placeholder.
//!
//! Error handling
//! - IO errors and render errors return HTTP 500 with a short, plaintext message.
//! - Missing files return HTTP 404 and prefer `meta/404.html` if present.
//!
//! Performance notes
//! - Use `Cofg::new()` (cached config) in hot paths; avoid forcing reloads here.
//! - Markdown files are parsed on each request. For large/unchanged files, consider a future
//!   (path, mtime) HTML cache (see `performance-cache.md`).
//!
//! Path resolution & security
//! - Disk paths come from `req.cached_public_req_path(&Cofg::new())` and are based on
//!   `Cofg.public_path`.
//! - Known improvement: no strict canonical-prefix enforcement yet; add traversal hardening
//!   if serving untrusted content roots.
//!
//! Quick examples
//! - `/` → `public/index.html` if present; otherwise dynamic TOC → HTML template.
//! - `/docs/readme.md` → render Markdown with `path:".../docs/readme.md"` in context.
//! - `/assets/logo.png` → served as a static file.
//!
//! See also
//! - `src/http_ext.rs`: request-level caching helpers.
//! - `src/parser/{markdown.rs, templating.rs}`: Markdown → HTML fragment and template engine.
//! - `.github/copilot-instructions.md`: architecture and hot-reload notes.
//!
//! 中文速覽
//! - 路由：
//!   - `GET /`：若有 `public/index.html` 直接回傳；否則以 `get_toc` 產生 TOC，交給 `md2html`
//!     注入模板輸出。
//!   - `GET /{filename:.*}`：若檔案不存在→優先回 `meta/404.html`；若為 Markdown→讀檔 + `md2html`
//!     並注入 `path:<實際路徑>`；否則走靜態檔回應。
//! - 原因：將索引頁與一般檔案處理分離；一般路徑統一「Markdown 即時轉換」或「靜態檔」。
//! - 效能：`Cofg::new()` 走快取；Markdown 每請求解析，未來可用 (path, mtime) 快取。
//! - 安全：目前未強制 canonical prefix；若內容根不受信，建議新增 traversal 防護。

use std::{ fs::read_to_string, path::Path };

use actix_files::NamedFile;
use actix_web::{ http::header, mime, Responder };
use log::{ debug, error, warn };

use crate::{
  cofg::config::Cofg,
  http_ext::HttpRequestCachedExt as _,
  parser::{ markdown::get_toc, md2html },
};

/// return `500 INTERNAL_SERVER_ERROR` with header plaintext utf-8
fn server_error(err_text: String) -> actix_web::HttpResponse {
  actix_web::HttpResponseBuilder
    ::new(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    .insert_header(actix_web::http::header::ContentType::plaintext())
    .body(err_text)
}

/// Prefer meta/404.html if available, otherwise a plain-text 404 response.
async fn respond_404(req: &actix_web::HttpRequest) -> actix_web::HttpResponse {
  use actix_web::http::StatusCode;
  match actix_files::NamedFile::open_async("./meta/404.html").await {
    Ok(file) => {
      let mut res = file.into_response(req);
      res = res
        .customize()
        .append_header(header::ContentType(mime::TEXT_HTML_UTF_8))
        .with_status(StatusCode::NOT_FOUND)
        .respond_to(req)
        .map_into_boxed_body();
      res
    }
    Err(e) => {
      warn!("failed to open 404.html: {e}");
      actix_web::HttpResponseBuilder::new(StatusCode::NOT_FOUND).body("404 Not Found")
    }
  }
}

/// Render a markdown file into HTML via `md2html` and return an HTTP response.
/// Render a markdown file into HTML via `md2html` and return an HTTP response.
///
/// Params:
/// - `req_path`: absolute canonical path to the markdown file
/// - `public_root`: absolute canonical root of `public_path` for fast `strip_prefix`
/// - `c`: read-only server configuration
///
/// Note: `public_root` is passed from caller to avoid recomputing `canonicalize()` in hot paths.
fn render_markdown_to_html_response(
  req_path: &Path,
  public_root: &Path,
  c: &Cofg
) -> actix_web::HttpResponse {
  use actix_web::{ HttpResponseBuilder, http::StatusCode };
  match read_to_string(req_path) {
    Ok(file) => {
      let out = crate::parser::md2html(
        file,
        c,
        vec![
          format!(
            "path:{}",
            req_path
              .strip_prefix(public_root)
              .unwrap_or_else(|e| {
                warn!("{e}");
                req_path
              })
              .display()
          )
        ]
      );
      match out {
        Ok(html) =>
          HttpResponseBuilder::new(StatusCode::OK)
            .append_header(header::ContentType(mime::TEXT_HTML_UTF_8))
            .body(html),
        Err(err) => {
          warn!("{err}");
          server_error(err.to_string())
        }
      }
    }
    Err(err) => {
      warn!("{err}: {}", err.kind());
      server_error(err.to_string())
    }
  }
}

#[actix_web::get("/{filename:.*}")]
/// Fallback handler for any path (captures `/{filename:.*}`) serving either a rendered markdown
/// or static file; returns custom 404 page if missing.
///
/// Flow:
/// 1. Resolve absolute disk path under `public_path`
/// 2. If missing → attempt meta/404.html else plain text 404
/// 3. If markdown → read + `md2html` with `path:` context
/// 4. Else stream static file
///
/// WHY: Unify file resolution & markdown rendering into one route while keeping index logic
/// separate for TOC special-case.
/// 中文：統一路徑處理；Markdown 即時轉換，其餘走靜態檔。
pub(crate) async fn main_req(req: actix_web::HttpRequest) -> impl actix_web::Responder {
  use actix_web::HttpResponseBuilder;

  debug!("{req:#?}");

  let c = &Cofg::new();
  let public_path = &Path::new(&c.public_path).canonicalize().unwrap();
  // Resolve the target path under the configured public root.
  let req_path = req.cached_public_req_path(c);
  debug!("req_path: {}", req_path.display());
  if !req_path.exists() {
    debug!("{}:!exists", req_path.display());
    return respond_404(&req).await;
  }
  let req_path = &(match req_path.canonicalize() {
    Ok(p) => p,
    Err(e) => {
      warn!("Failed to canonicalize req_path: {e}");
      req_path
    }
  });
  let req_strip_prefix_path = match req_path.strip_prefix(public_path) {
    Ok(p) => p,
    Err(e) => {
      warn!("{e}");
      req_path
    }
  };

  if req.cached_is_markdown(c) {
    debug!("is md");
    // Render Markdown to HTML and return.
    render_markdown_to_html_response(req_path, public_path, c)
  } else if req_path.is_file() {
    debug!("no md");
    match NamedFile::open_async(req_path).await {
      Ok(file) => file.into_response(&req),
      Err(err) => {
        warn!("{err}: {}", err.kind());
        server_error(err.to_string())
      }
    }
  } else if req_path.is_dir() {
    debug!("is dir");
    let toc = get_toc(req_path, c, Some(req_strip_prefix_path.to_string_lossy().to_string()));
    if let Ok(v) = toc {
      let r = md2html(
        v,
        c,
        vec![format!("path:toc:{}", req_strip_prefix_path.to_string_lossy()).to_string()]
      );
      if let Ok(html) = r {
        HttpResponseBuilder::new(actix_web::http::StatusCode::OK)
          .append_header(header::ContentType(mime::TEXT_HTML_UTF_8))
          .body(html)
      } else {
        let err = r.err().unwrap();
        warn!("{err}");
        server_error(err.to_string())
      }
    } else {
      let err = toc.err().unwrap();
      warn!("{err}");
      server_error(err.to_string())
    }
  } else {
    error!("{}: not file and dir", req_strip_prefix_path.display());
    server_error(format!("{}: not file and dir", req_strip_prefix_path.display()))
  }
}

#[actix_web::get("/", name = "index")]
/// Index route: serve `public/index.html` if present else dynamic TOC rendered via markdown.
///
/// WHY: Avoid forcing a pre-generated index; dynamic TOC ensures consistency with current files.
/// Using TOC only when index missing allows users to override with custom landing page.
/// 中文：若存在自定義 index 則優先；否則即時產生 TOC 提供導覽。
pub(crate) async fn index(_: actix_web::HttpRequest) -> impl actix_web::Responder {
  use actix_web::HttpResponseBuilder;

  let c = &Cofg::new();

  let index_file = Path::new(&c.public_path).join("index.html");
  if index_file.exists() {
    let f = read_to_string(index_file);
    if let Ok(value) = f {
      debug!("index exists=>show file");
      HttpResponseBuilder::new(actix_web::http::StatusCode::OK)
        .append_header(header::ContentType(mime::TEXT_HTML_UTF_8))
        .body(value)
    } else {
      let err = f.err().unwrap();
      warn!("{err}");
      server_error(err.to_string())
    }
  } else {
    debug!("index !exists=>get toc");
    let toc = get_toc(
      &Path::new(&c.public_path).canonicalize().unwrap(),
      c,
      Some("index".to_string())
    );
    if let Ok(v) = toc {
      let r = md2html(v, c, vec!["path:toc:index".to_string()]);
      if let Ok(html) = r {
        HttpResponseBuilder::new(actix_web::http::StatusCode::OK)
          .append_header(header::ContentType(mime::TEXT_HTML_UTF_8))
          .body(html)
      } else {
        let err = r.err().unwrap();
        warn!("md2html: {err}");
        server_error(err.to_string())
      }
    } else {
      let err = toc.err().unwrap();
      warn!("get_toc: {err}");
      server_error(err.to_string())
    }
  }
}
