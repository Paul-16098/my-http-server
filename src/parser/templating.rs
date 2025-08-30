//! templating

use mystical_runic::{ TemplateEngine, TemplateContext };

#[inline]
pub(crate) fn get_context(c: &crate::cofg::Cofg) -> TemplateContext {
  let mut context = TemplateContext::new();
  context.set_string("server-version", env!("CARGO_PKG_VERSION"));
  if let Some(templating_value) = &c.templating.value {
    templating_value.iter().for_each(|(k, v)| {
      context.set_string(k, v);
    });
  }

  context
}
#[inline]
pub(crate) fn get_engine(c: &crate::cofg::Cofg) -> TemplateEngine {
  let mut engine = TemplateEngine::new("./meta");
  engine.enable_bytecode_cache(true);
  if c.templating.hot_reload {
    engine.enable_hot_reload();
  }
  engine
}
