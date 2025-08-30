//! templating

use mystical_runic::{ TemplateEngine, TemplateContext };

#[inline]
pub(crate) fn get_context() -> TemplateContext {
  let mut context = TemplateContext::new();
  context.set_string("server-version", env!("CARGO_PKG_VERSION"));
  context
}
#[inline]
pub(crate) fn get_engine() -> TemplateEngine {
  TemplateEngine::new("./meta")
}
