//! Markdown related helpers
//!
//! WHY: Keep bulk utilities (batch convert, TOC generation) separated from request path so server
//! startup & per-request logic remain lean. Functions prefixed with '_' are tooling helpers not
//! wired into HTTP routes by default.
//!
//! 中文：將批次轉換/TOC 邏輯與線上請求分離，保持主流程簡潔；底線開頭為工具函式。

use std::collections::BTreeMap;
use std::path::{ Path, PathBuf };
use std::sync::{ Mutex, OnceLock };
use std::time::SystemTime;

use log::{ debug, trace };
use wax::Glob;

use crate::error::AppResult;
use crate::{ cofg::config::Cofg, error };

const NON_ALPHANUMERIC: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC.remove(
  b'/'
);

#[derive(Default, Debug)]
struct TocNode {
  children: BTreeMap<String, TocNode>,
}

fn emit_toc(node: &TocNode, prefix: &mut Vec<String>, out: &mut String, depth: usize) {
  for (name, child) in &node.children {
    let path = if prefix.is_empty() {
      name.clone()
    } else {
      format!("{}/{}", prefix.join("/"), name)
    };
    let indent = " ".repeat(depth * 4);
    trace!("emit_toc: node={node:?};prefix={prefix:#?};out={out};depth={depth}");
    out.push_str(
      &format!(
        "{indent}- [{}]({})\n",
        name,
        percent_encoding::utf8_percent_encode(&path.replace("\\", "/"), NON_ALPHANUMERIC)
      )
    );

    prefix.push(name.clone());
    emit_toc(child, prefix, out, depth + 1);
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
pub(crate) fn get_toc(root_path: &Path, c: &Cofg, title: Option<String>) -> AppResult<String> {
  debug!("root:{}", root_path.display());
  let public_path = &Path::new(&c.public_path).canonicalize()?;
  let root_path = &root_path.canonicalize()?;

  // Simple TOC memoization: key by (abs_dir, last_modified, title)
  // Invalidation risk is minimized by using directory metadata modified time when available.
  // If retrieval fails, we skip cache and compute fresh.
  let title_key = title.clone();
  if c.cache.enable_toc {
    let cap = std::num::NonZeroUsize::new(c.cache.toc_capacity.max(1)).unwrap();
    let cache = TOC_CACHE.get_or_init(|| Mutex::new(lru::LruCache::new(cap)));
    if
      let Some(hit) = cache
        .lock()
        .ok()
        .and_then(|mut cache| {
          TocCacheKey::from_dir(root_path, title_key.clone()).and_then(|k| cache.get(&k).cloned())
        })
    {
      return Ok(hit);
    }
  }

  let mut toc_str = format!("# {}\n\n", title.unwrap_or("toc".to_string()));
  // Build a tree of path components for stable, de-duplicated recursive output
  let mut root: TocNode = TocNode::default();

  // 將 HashSet 轉為排好序的 Vec 並 join 成 glob 模式（修正原先 Vec::from(c.toc.ext) 編譯錯誤）
  let exts: Vec<String> = c.toc.ext.iter().cloned().collect();
  let glob_pattern = format!("**/*.{{{}}}", exts.join(","));

  for entry in Glob::new(&glob_pattern)?.walk(root_path) {
    let entry = entry?;
    let path = entry.path().canonicalize()?.strip_prefix(root_path)?.to_path_buf();
    debug!("path: {}", path.display());

    // Skip entries matching any ignore token
    let path_str = path.to_string_lossy();
    if c.toc.ig.iter().any(|ele| path_str.contains(ele)) {
      debug!("continue");
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
  prefix.push(root_path.strip_prefix(public_path)?.to_string_lossy().into_owned());
  emit_toc(&root, &mut prefix, &mut toc_str, 0);
  // Store into cache on success
  if
    c.cache.enable_toc &&
    let (Some(cell), Some(k)) = (TOC_CACHE.get(), TocCacheKey::from_dir(root_path, title_key)) &&
    let Ok(mut cache) = cell.lock()
  {
    cache.put(k, toc_str.clone());
  }
  Ok(toc_str)
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

// --------------------
// Lightweight caches
// --------------------

// TOC cache keyed by (dir, dir_modified_ts, title). Bounded via LRU to prevent unbounded growth.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub(crate) struct TocCacheKey {
  pub(crate) dir: PathBuf,
  // coarse invalidation guard; fallback to 0 when metadata not available
  pub(crate) modified_unix_secs: u64,
  pub(crate) title: Option<String>,
}

impl TocCacheKey {
  pub(crate) fn from_dir(dir: &Path, title: Option<String>) -> Option<Self> {
    let modified_unix_secs = std::fs
      ::metadata(dir)
      .and_then(|m| m.modified())
      .ok()
      .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
      .map(|d| d.as_secs())
      .unwrap_or(0);
    Some(Self {
      dir: dir.to_path_buf(),
      modified_unix_secs,
      title,
    })
  }
}

static TOC_CACHE: OnceLock<Mutex<lru::LruCache<TocCacheKey, String>>> = OnceLock::new();
