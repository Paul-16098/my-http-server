use std::{fs::read_to_string, path::Path};

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
    debug!("public_path={}", public_path.display());

    // Resolve the target path under the configured public root.
    // path is "/" -> ""
    // WHY: 8c648dc7d9dcbf6769238ead8810aa7f324aaf7d
    let filename_str = if req.path() == "/" {
        ".".to_string()
    } else {
        format!(".{}", req.path())
    };
    debug!("filename_str={}", filename_str);
    let req_path_buf = public_path.join(Path::new(&filename_str));
    debug!("req_path_buf={}", req_path_buf.display());

    let req_path = &(match req_path_buf.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to canonicalize req_path: {e}");
            req_path_buf
        }
    });
    let req_strip_prefix_path = match req_path.strip_prefix(public_path) {
        Ok(p) => p,
        Err(e) => {
            error!(
                "{}: is a traversal dotdot attack? {req_path:?} to {public_path:?}",
                e
            );
            return actix_web::HttpResponseBuilder::new(actix_web::http::StatusCode::BAD_REQUEST)
                .body("No traversal dotdot attack allowed");
        }
    };

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
