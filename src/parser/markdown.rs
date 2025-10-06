//! Markdown related helpers
//!
//! WHY: Keep bulk utilities (batch convert, TOC generation) separated from request path so server
//! startup & per-request logic remain lean. Functions prefixed with '_' are tooling helpers not
//! wired into HTTP routes by default.
//!
//! 中文：將批次轉換/TOC 邏輯與線上請求分離，保持主流程簡潔；底線開頭為工具函式。

use std::{ fs::{ read_to_string, write }, path::Path };
use std::collections::BTreeMap;

use wax::Glob;

use crate::{ cofg::Cofg, error, parser::md2html };
use crate::error::{ AppResult, AppError };

/// Batch convert every `**/*.{md,markdown}` under `public_path` into `.html` siblings.
///
/// WHY: Optional build-time / manual utility; not invoked during normal server runtime to avoid
/// upfront cost. Keeps `md2html` itself pure so we can reuse it in both paths.
/// 中文：提供選擇性批次轉換工具，不影響伺服器啟動與即時渲染流程。
pub(crate) fn _md2html_all() -> AppResult<()> {
  let md_files = Glob::new("**/*.{md,markdown}")?;
  let cfg = crate::cofg::Cofg::new(); // returns cached config (not a fresh instance)
  let public_path = &cfg.public_path.clone();
  for entry in md_files.walk(public_path) {
    let entry = entry?; // WalkError already converted by ? via AppError
    let path = entry.path().to_path_buf();
    let out_path_obj = path.with_extension("html");
    write(
      &out_path_obj,
      md2html(
        read_to_string(&path)?,
        &cfg,
        vec![format!("path:{}", out_path_obj.strip_prefix(public_path).unwrap().display())]
      )?
    )?;
  }
  Ok(())
}

const NON_ALPHANUMERIC: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC.remove(
  b'/'
);

#[derive(Default)]
struct TocNode {
  children: BTreeMap<String, TocNode>,
}

fn emit_toc(
  node: &TocNode,
  prefix: &mut Vec<String>,
  out: &mut String,
  depth: usize,
  encode: &dyn Fn(&str) -> String
) {
  for (name, child) in &node.children {
    let path = if prefix.is_empty() {
      name.clone()
    } else {
      format!("{}/{}", prefix.join("/"), name)
    };
    let indent = " ".repeat(depth * 4);
    out.push_str(&format!("{indent}- [{}]({})\n", name, encode(&path)));

    prefix.push(name.clone());
    emit_toc(child, prefix, out, depth + 1, encode);
    prefix.pop();
  }
}

/// Generate an in-memory Markdown TOC listing files with configured extensions under `public_path`.
///
/// Each entry becomes `- [stem](percent-encoded-path)`; non-alphanumeric chars percent-encoded
/// except '/'. Base directory is `toc.path`'s parent to allow placing TOC inside subfolder.
///
/// WHY: On-demand generation avoids stale TOC and eliminates pre-bake step. Lightweight glob walk
/// acceptable since `/` root requests are comparatively infrequent.
/// 中文：即時產生，避免 TOC 與檔案狀態不一致；根據 toc.path 上層資料夾決定掃描基準。
pub(crate) fn get_toc(path: &Path, c: &Cofg, title: Option<String>) -> AppResult<String> {
  let out_path = &Path::new(path).join(std::ops::Deref::deref(&c.toc.path));
  let out_dir = &out_path
    .parent()
    .ok_or_else(|| AppError::Other("toc path has no parent".into()))?
    .canonicalize()?;

  let mut toc_str = format!("# {}\n\n", title.unwrap_or("toc".to_string()));
  // Build a tree of path components for stable, de-duplicated recursive output
  let mut root: TocNode = TocNode::default();
  for entry in Glob::new(&format!("**/*.{{{}}}", c.toc.ext.join(",")))?.walk(path) {
    let entry = entry?;
    let path = entry
      .path()
      .canonicalize()?
      .strip_prefix(out_dir)
      .map_err(|e| AppError::Other(format!("strip_prefix: {e}")))?
      .to_path_buf();

    // Skip entries matching any ignore token
    let path_str = path.to_string_lossy();
    if c.toc.ig.iter().any(|ele| path_str.contains(ele)) {
      continue;
    }

    let comps: Vec<String> = path
      .components()
      .map(|c| c.as_os_str().to_string_lossy().to_string())
      .collect();
    let mut cur = &mut root;
    for part in comps {
      cur = cur.children.entry(part).or_default();
    }
  }

  // Emit recursively for arbitrary depth
  let mut prefix: Vec<String> = Vec::new();
  emit_toc(&root, &mut prefix, &mut toc_str, 0, &encode_path);
  Ok(toc_str)
}

/// Module-level encode helper for links
fn encode_path(s: &str) -> String {
  percent_encoding::utf8_percent_encode(&s.replace("\\", "/"), NON_ALPHANUMERIC).to_string()
}

/// Parse raw markdown into AST using markdown_ppp with default config.
///
/// WHY: Encapsulate parser selection & config; caller obtains structured AST for potential future
/// analysis (currently only rendered directly). Keeps `md2html` simpler.
/// 中文：抽離解析步驟，未來若需 AST 進階處理可在此擴充。
pub(crate) fn parser_md(input: String) -> error::AppResult<markdown_ppp::ast::Document> {
  use markdown_ppp::parser::parse_markdown;
  // 內部需要 clone config 給 parser，但外部呼叫時可傳參考，避免重複 clone
  Ok(
    parse_markdown(
      markdown_ppp::parser::MarkdownParserState::with_config(
        markdown_ppp::parser::config::MarkdownParserConfig::default()
      ),
      &input
    )?
  )
}
