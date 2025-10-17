//! parser
//!
//! WHY: 提供「Markdown → HTML 片段 → 套模板」的單一進入點（`md2html`），
//! 把快取、模板熱重載與 Context 組裝細節封裝在內部模組（`templating`, `markdown`）中，
//! 呼叫端只需關注輸入字串與可選的模板變數清單。

use crate::parser::templating::set_context_value;
use std::hash::{ Hash, Hasher };
use std::path::{ Path, PathBuf };
use std::sync::{ Mutex, OnceLock };

pub(crate) mod markdown;
pub(crate) mod templating;

/// Convert a single markdown string into full HTML page via template `html-t` (file: `./meta/html-t.hbs`).
///
/// Steps:
/// 1. Acquire (or rebuild) template engine
/// 2. Build fresh context (server + configured vars)
/// 3. Apply extra template_data_list entries (e.g. `path:...`)
/// 4. Parse markdown → AST → HTML body
/// 5. Inject `body` then render compiled template
///
/// WHY: Keep side effects (engine caching, context assembly) localized while exposing a pure-ish
/// interface to callers. Accepts owned `md` so upstream can cheaply `read_to_string` and transfer
/// ownership without clone.
/// 中文：集中渲染步驟，讓呼叫端只需提供字串與附加變數；擁有字串避免多餘 clone。
///
/// Contract / 契約（重要行為與邊界）
/// - Inputs:
///   - `md`: 原始 Markdown（UTF-8）。不做串流；一次性解析。
///   - `c`: 讀取模板設定與 hot reload 旗標；實際模板引擎取得見 `templating::get_engine`。
///   - `template_data_list`: 以 `name:value`（含 `name:env:ENV`）格式注入 Context；
///     智慧型別推斷順序為 bool → i64 → string；無冒號或格式不正者將被忽略。
///     後加入的條目會覆寫同名既有值（包含由設定檔注入者）。
/// - Output: 以邏輯名 `html-t` 渲染完成的完整 HTML 字串；Context 會包含：
///   - 由 `get_context` 注入的內建鍵：`server-version`
///   - 本函式注入的 `body`：Markdown 轉出的 HTML 片段
/// - Errors: 可能來自
///   - Markdown 解析失敗（語法錯誤或非預期情形）
///   - 模板檔案註冊/解析失敗（檔案缺失或模板語法錯誤）
///   - 模板渲染失敗（缺鍵/型別不符等）→ 包裝為 `AppError::RenderError`
/// - Side effects:
///   - 首次渲染若引擎尚未註冊 `html-t`，會以 `./meta/html-t.hbs` 進行註冊（讀檔）。
///   - 以 `trace` 層級輸出 AST（大型文件可能產生大量日誌）。
/// - Perf/Security notes:
///   - 正常模式引擎為快取重用；`hot_reload=true` 時每請求重建以反映模板改動。
///   - 渲染依賴本機模板檔案路徑；如內容根不可信，請配合上游路徑檢查避免 traversal。
pub(crate) fn md2html(
  md: String,
  c: &crate::cofg::config::Cofg,
  template_data_list: Vec<String>
) -> crate::error::AppResult<String> {
  // Compute a small, stable hash of the extra template data to separate cache entries
  // when callers inject different variables (e.g., theming flags). This prevents
  // incorrect reuse across differing contexts.
  let mut ctx_hasher = std::collections::hash_map::DefaultHasher::new();
  for s in template_data_list.iter() {
    s.hash(&mut ctx_hasher);
  }
  let template_ctx_hash = ctx_hasher.finish();

  // Attempt to extract a disk-relative path from template data: expects a key "path:<rel>"
  // or "path:toc:<dir>". Only file paths (non-TOC) participate in the HTML cache.
  let rel_path_opt: Option<String> = template_data_list
    .iter()
    .find_map(|s| s.strip_prefix("path:"))
    .map(|v| v.to_string());

  let is_toc_context = rel_path_opt
    .as_deref()
    .map(|v| v.starts_with("toc:"))
    .unwrap_or(false);

  let mut maybe_cache_key: Option<MdCacheKey> = None;
  if !is_toc_context && let Some(rel) = rel_path_opt.as_ref() {
    // Build absolute path under public root
    let abs = Path::new(&c.public_path).join(rel);
    if let Ok(meta) = std::fs::metadata(&abs) {
      let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
      let size = meta.len();
      let template_mtime = template_file_mtime();
      maybe_cache_key = Some(MdCacheKey {
        abs_path: abs,
        mtime_secs: mtime,
        size,
        template_mtime_secs: template_mtime,
        template_ctx_hash,
      });
    }
  }

  if c.cache.enable_html && let Some(key) = maybe_cache_key.as_ref() {
    let cap = std::num::NonZeroUsize::new(c.cache.html_capacity.max(1)).unwrap();
    let cache = MD_CACHE.get_or_init(|| { Mutex::new(lru::LruCache::new(cap)) });
    if let Ok(mut guard) = cache.lock() && let Some(html) = guard.get(key) {
      return Ok(html.clone());
    }
  }

  // let c = &crate::cofg::Cofg::new();
  let mut engine = templating::get_engine(c)?;
  let mut context = templating::get_context(c);
  // NOTE: 後寫優先（呼叫端提供者可覆寫設定注入的鍵）。
  for template_data in template_data_list {
    set_context_value(&mut context, &template_data);
  }
  // Lazy 註冊模板：避免在未使用時就讀檔；同時配合 hot reload（引擎重建後將再次註冊）。
  if !engine.has_template("html-t") {
    engine.register_template_file("html-t", "./meta/html-t.hbs")?;
  }

  let mut ast = markdown::parser_md(md)?;
  // PERF: 只在 trace 開啟時輸出 AST；大型 Markdown 可能造成龐大日誌量。
  log::trace!("ast={ast:#?}");
  if cfg!(feature = "github_emojis") {
    // 將 emojis.json 解析為 HashMap，並以 OnceLock 快取以避免重複解析
    static EMOJI_MAP: OnceLock<std::collections::HashMap<String, String>> = OnceLock::new();
    let emojis = EMOJI_MAP.get_or_init(|| {
      let data = include_str!("./../../emojis.json");
      match serde_json::from_str::<std::collections::HashMap<String, String>>(data) {
        Ok(map) => map,
        Err(e) => {
          log::warn!("failed to parse emojis.json: {}", e);
          std::collections::HashMap::new()
        }
      }
    });

    // 輕量級 :shortcode: 掃描與替換
    struct ReplaceGithubEmojis<'a> {
      emojis: &'a std::collections::HashMap<String, String>,
    }
    impl<'a> markdown_ppp::ast_transform::Transformer for ReplaceGithubEmojis<'a> {
      fn transform_inline(
        &mut self,
        inline: markdown_ppp::ast::Inline
      ) -> markdown_ppp::ast::Inline {
        match inline {
          markdown_ppp::ast::Inline::Text(code) => {
            let mut text = code;
            for (k, v) in self.emojis.iter() {
              let pat = format!(":{k}:");
              if text.contains(&pat) {
                let rep = format!(
                  r#"<img class="emoji" alt="{pat}" src="{v}" style="width: 1em;">"#
                );
                // 以 &str 傳入 replace，並指派回字串
                text = text.replace(&pat, &rep);
              }
            }
            markdown_ppp::ast::Inline::Html(text)
          }
          other => self.walk_transform_inline(other),
        }
      }
    }

    ast = markdown_ppp::ast_transform::Transform::transform_with(ast, ReplaceGithubEmojis {
      emojis,
    });
  }
  log::trace!("ast={ast:#?}");
  let html = markdown_ppp::html_printer::render_html(
    &ast,
    markdown_ppp::html_printer::config::Config::default()
  );

  // Contract: 模板預期取得 `body` 作為主要內容插槽。
  context.data_mut()["body"] = handlebars::JsonValue::String(html);
  match engine.render_with_context("html-t", &context) {
    Ok(o) => {
      if
        c.cache.enable_html &&
        let Some(key) = maybe_cache_key &&
        let Some(cell) = MD_CACHE.get() &&
        let Ok(mut guard) = cell.lock()
      {
        guard.put(key, o.clone());
      }
      Ok(o)
    }
    Err(o) => {
      log::error!("md2html:{}", o);
      Err(crate::error::AppError::RenderError(o))
    }
  }
}

// --------------------
// Rendered HTML cache for Markdown files
// --------------------
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct MdCacheKey {
  abs_path: PathBuf,
  mtime_secs: u64,
  size: u64,
  template_mtime_secs: u64,
  template_ctx_hash: u64,
}

fn template_file_mtime() -> u64 {
  std::fs
    ::metadata("./meta/html-t.hbs")
    .and_then(|m| m.modified())
    .ok()
    .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
    .map(|d| d.as_secs())
    .unwrap_or(0)
}

static MD_CACHE: OnceLock<Mutex<lru::LruCache<MdCacheKey, String>>> = OnceLock::new();
