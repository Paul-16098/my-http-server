//! templating

use mystical_runic::{ TemplateEngine, TemplateContext };

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
pub(crate) fn get_engine(c: &crate::cofg::Cofg) -> TemplateEngine {
  let mut engine = TemplateEngine::new("./meta");
  engine.enable_bytecode_cache(true);
  if c.templating.hot_reload {
    engine.enable_hot_reload();
  }
  engine
}
