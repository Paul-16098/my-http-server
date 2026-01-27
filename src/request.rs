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
//! - Markdown files are parsed on each request (HTML caching disabled / removed).
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

use std::{fs::read_to_string, path::Path, process::exit};

use actix_files::NamedFile;
use actix_web::{Responder, http::header, mime};
use log::{debug, error, warn};

use crate::{
    cofg::config::Cofg,
    error::AppResult,
    parser::{markdown::get_toc, md2html},
};

/// return `500 INTERNAL_SERVER_ERROR` with header plaintext utf-8
pub(crate) fn server_error(err_text: String) -> actix_web::HttpResponse {
    actix_web::HttpResponseBuilder::new(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
        .insert_header(actix_web::http::header::ContentType::plaintext())
        .body(err_text)
}

/// Prefer configured 404 page if available, otherwise a plain-text 404 response.
/// Respects configuration layering: checks local path first, then XDG config directory.
async fn respond_404(req: &actix_web::HttpRequest) -> actix_web::HttpResponse {
    use actix_web::http::StatusCode;
    let c = &Cofg::get(false);
    let page_404_path = c.resolve_page_404_path();
    match actix_files::NamedFile::open_async(&page_404_path).await {
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
            warn!("failed to open {}: {e}", page_404_path.display());
            actix_web::HttpResponseBuilder::new(StatusCode::NOT_FOUND).body("404 Not Found")
        }
    }
}

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
    c: &Cofg,
) -> AppResult<actix_web::HttpResponse> {
    use actix_web::{HttpResponseBuilder, http::StatusCode};
    match read_to_string(req_path) {
        Ok(md_source) => {
            // Render Markdown directly into the HTML template with path context.
            let rel = req_path
                .strip_prefix(public_root)
                .unwrap_or_else(|e| {
                    warn!("{e}");
                    req_path
                })
                .to_path_buf();

            match md2html(md_source, c, vec![format!("path:{}", rel.display())]) {
                Ok(html) => Ok(HttpResponseBuilder::new(StatusCode::OK)
                    .append_header(header::ContentType(mime::TEXT_HTML_UTF_8))
                    .body(html)),
                Err(err) => {
                    warn!("{err}");
                    Ok(server_error(err.to_string()))
                }
            }
        }
        Err(err) => {
            warn!("{err}: {}", err.kind());
            Ok(server_error(err.to_string()))
        }
    }
}

/// Render a directory TOC via `get_toc` + `md2html` into the HTML template shell.
///
/// Params:
/// - `dir_abs`: absolute canonical directory path whose TOC will be generated
/// - `ctx_label`: logical label for both TOC title and the `path:toc:<label>` context value
/// - `c`: read-only server configuration
fn render_toc_to_html_response(
    dir_abs: &Path,
    ctx_label: &str,
    c: &Cofg,
) -> actix_web::HttpResponse {
    use actix_web::{HttpResponseBuilder, http::StatusCode};
    let label = if ctx_label.is_empty() { "?" } else { ctx_label };
    debug!("{}", label);
    let toc = get_toc(dir_abs, c, Some(label.to_string()));
    match toc {
        Ok(v) => {
            let r = md2html(v, c, vec![format!("path:toc:{label}")]);
            match r {
                Ok(html) => HttpResponseBuilder::new(StatusCode::OK)
                    .append_header(header::ContentType(mime::TEXT_HTML_UTF_8))
                    .body(html),
                Err(err) => {
                    warn!("{err}");
                    server_error(err.to_string())
                }
            }
        }
        Err(err) => {
            warn!("{err}");
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
    debug!("{req:?}");

    let c = &Cofg::get(false);
    let public_path = &Path::new(&c.public_path)
        .canonicalize()
        .unwrap_or_else(|e| {
            warn!(
                "Failed to canonicalize public_path: {} = {e}",
                c.public_path
            );
            Path::new(&c.public_path).to_path_buf()
        });
    // Resolve the target path under the configured public root.

    // path is "/" -> ""
    // WHY: 8c648dc7d9dcbf6769238ead8810aa7f324aaf7d
    let filename_str = if req.path() == "/" {
        "".to_string()
    } else {
        req.path().to_string()
    };
    let req_path_buf = Path::new(&c.public_path).join(filename_str);
    let req_path = &req_path_buf;
    if req_path == public_path {
        let index_file = public_path.join("index.html");
        if index_file.exists() {
            let f = read_to_string(index_file);
            match f {
                Ok(value) => {
                    debug!("index exists=>show file");
                    return actix_web::HttpResponseBuilder::new(actix_web::http::StatusCode::OK)
                        .append_header(header::ContentType(mime::TEXT_HTML_UTF_8))
                        .body(value);
                }
                Err(err) => {
                    warn!("{err}");
                    return server_error(err.to_string());
                }
            }
        }
    }
    if !req_path.exists() {
        debug!("{}:!exists", req_path.display());
        return respond_404(&req).await;
    }
    if ["cofg.yaml", ".gitignore", "Cargo.toml"]
        .iter()
        .any(|f| req_path.ends_with(f))
    {
        error!(
            "!!! Access to restricted file: {} by {:?}",
            req_path.display(),
            req
        );
    }
    let req_path = &(match req_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to canonicalize req_path: {e}");
            req_path.to_path_buf()
        }
    });
    let req_strip_prefix_path = match req_path.strip_prefix(public_path) {
        Ok(p) => p,
        Err(e) => {
            debug!("req_path: {}", req_path.display());
            debug!("public_path: {}", public_path.display());
            warn!("{e}");
            exit(1)
        }
    };

    let is_md = req_path.extension().and_then(|v| v.to_str()) == Some("md");
    if is_md {
        debug!("is md");
        // Render Markdown to HTML and return.
        match render_markdown_to_html_response(req_path, public_path, c) {
            Ok(res) => res,
            Err(err) => server_error(err.to_string()),
        }
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
        let label = req_strip_prefix_path.to_string_lossy();
        if req_path == public_path {
            // if is index
            render_toc_to_html_response(req_path, "index", c)
        } else {
            render_toc_to_html_response(req_path, &label, c)
        }
    } else {
        error!("{}: not file and dir", req_strip_prefix_path.display());
        server_error(format!(
            "{}: not file and dir",
            req_strip_prefix_path.display()
        ))
    }
}
