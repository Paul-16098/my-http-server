//! templating

use mystical_runic::{ TemplateEngine, TemplateContext };
use once_cell::sync::OnceCell;
use std::sync::RwLock;

pub(crate) fn get_context(c: &crate::cofg::Cofg) -> TemplateContext {
  let mut context = TemplateContext::new();
  context.set_string("server-version", env!("CARGO_PKG_VERSION"));
  if let Some(raw_str) = &c.templating.value {
    for data in raw_str {
      let d: Vec<String> = data
        .split(":")
        .map(|s| s.trim().to_string())
        .collect();
      let (name, value) = (&d[0], &d[1]);
      if let Ok(tf) = value.parse() {
        context.set_bool(name, tf);
      } else if let Ok(num) = value.parse() {
        context.set_number(name, num);
      } else {
        context.set_string(name, value);
      }
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
