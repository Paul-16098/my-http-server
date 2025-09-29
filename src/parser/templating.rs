//! Template engine helpers
//!
//! WHY: Isolate template engine initialization & context population so markdown → HTML pipeline
//! stays pure/minimal (`md2html`). Engine instance is cached via `OnceCell<RwLock<_>>` for reuse
//! (bytecode cache active) unless `hot_reload` signals we must reconstruct to pick up file edits.
//!
//! 中文：將模板引擎建構與上下文變數設定集中，讓 `md2html` 保持精簡；常態重用快取以利用
//! bytecode cache，僅在 hot_reload 啟用時每次重建以反映檔案修改。

use mystical_runic::{ TemplateEngine, TemplateContext };
use once_cell::sync::OnceCell;
use std::sync::RwLock;

/// Inject a single `name:value` pair (auto type inference) into template context.
///
/// Supported forms:
/// - `foo:true/false` → bool
/// - `foo:123` → number (i64)
/// - `foo:some string` → string
/// - `foo:env:ENV_NAME` → reads `ENV_NAME` then infers (bool/number/string)
///
/// WHY: Allow configuration-driven variable list (`templating.value`) without schema explosion.
/// Parsing kept intentionally small (no floats) for predictability.
///
/// 中文：透過簡單 `name:value` 規則（含 `env:` 展開）注入模板變數，避免為多個旗標打造冗長設定欄位。
pub(crate) fn set_context_value(context: &mut TemplateContext, data: &str) {
  // Split only at the first ':'; skip malformed entries
  if let Some((name_raw, value_raw)) = data.split_once(':') {
    let name = name_raw.trim();
    let mut value = value_raw.trim().to_string();
    if name.is_empty() {
      return;
    }

    if
      value.starts_with("env:") &&
      let Some((_, env)) = value.split_once(":") &&
      let Ok(v) = std::env::var(env)
    {
      value = v;
    }

    if let Ok(tf) = value.parse::<bool>() {
      context.set_bool(name, tf);
      return;
    }

    // Parse numbers (i64 only)
    if let Ok(num) = value.parse::<i64>() {
      context.set_number(name, num);
    } else {
      context.set_string(name, &value);
    }
  } // else: no ':', ignore entry safely
}

/// Build a fresh template context with server metadata and configured variables.
///
/// Always includes `server-version`. Then folds `templating.value` items through
/// `set_context_value` to infer types.
///
/// WHY: Decouple config parsing from render path; context creation is cheap and explicit instead
/// of sharing mutable state across renders.
/// 中文：每次渲染建立獨立 context，避免共享可變狀態；同時注入版本資訊與設定變數。
pub(crate) fn get_context(c: &crate::cofg::Cofg) -> TemplateContext {
  let mut context = TemplateContext::new();
  context.set_string("server-version", env!("CARGO_PKG_VERSION"));
  if let Some(raw_str) = &c.templating.value {
    for data in raw_str {
      set_context_value(&mut context, data);
    }
  }

  context
}
static ENGINE: OnceCell<RwLock<TemplateEngine>> = OnceCell::new();

/// Retrieve (or rebuild under hot reload) the template engine.
///
/// On first call, initializes with bytecode cache (and optional hot reload). If `hot_reload` is
/// true, each call reconstructs a new engine to ensure latest template file contents are used.
///
/// WHY: Development ergonomics—trade small rebuild cost for immediacy when editing templates.
/// Production (no hot_reload) benefits from stable cached engine with bytecode reuse.
/// 中文：hot_reload 時每次重建以反映檔案變更；否則重用快取獲得效能與 bytecode 優勢。
pub(crate) fn get_engine(c: &crate::cofg::Cofg) -> TemplateEngine {
  let cell = ENGINE.get_or_init(|| {
    let mut e = TemplateEngine::new("./meta");
    e.enable_bytecode_cache(true);
    if c.templating.hot_reload {
      e.enable_hot_reload();
    }
    RwLock::new(e)
  });

  // if hot_reload enabled always recreate fresh engine (template file may change)
  if c.templating.hot_reload && let Ok(mut w) = cell.write() {
    let mut e = TemplateEngine::new("./meta");
    e.enable_bytecode_cache(true);
    e.enable_hot_reload();
    *w = e;
    return w.clone();
  }

  cell
    .read()
    .map(|e| e.clone())
    .unwrap_or_else(|_| {
      let mut e = TemplateEngine::new("./meta");
      e.enable_bytecode_cache(true);
      if c.templating.hot_reload {
        e.enable_hot_reload();
      }
      e
    })
}
