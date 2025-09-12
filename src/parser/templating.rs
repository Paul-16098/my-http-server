//! templating

use mystical_runic::{ TemplateEngine, TemplateContext };
use once_cell::sync::OnceCell;
use std::sync::RwLock;

pub(crate) fn set_context(context: &mut TemplateContext, data: &str) {
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

pub(crate) fn get_context(c: &crate::cofg::Cofg) -> TemplateContext {
  let mut context = TemplateContext::new();
  context.set_string("server-version", env!("CARGO_PKG_VERSION"));
  if let Some(raw_str) = &c.templating.value {
    for data in raw_str {
      set_context(&mut context, data);
    }
  }

  context
}
static ENGINE: OnceCell<RwLock<TemplateEngine>> = OnceCell::new();

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
