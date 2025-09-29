//! parser

use crate::parser::templating::set_context_value;

pub(crate) mod markdown;
pub(crate) mod templating;

/// Convert a single markdown string into full HTML page via template `html-t.templating`.
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
pub(crate) fn md2html(
  md: String,
  c: &crate::cofg::Cofg,
  template_data_list: Vec<String>
) -> crate::error::AppResult<String> {
  // let c = &crate::cofg::Cofg::new();
  let mut engine = templating::get_engine(c);
  let mut context = templating::get_context(c);
  for template_data in template_data_list {
    set_context_value(&mut context, &template_data);
  }

  let html_t = engine.compile_to_bytecode("html-t.templating")?;

  let ast = markdown::parser_md(md)?;
  log::trace!("ast={ast:#?}");
  let html = &markdown_ppp::html_printer::render_html(
    &ast,
    markdown_ppp::html_printer::config::Config::default()
  );

  context.set_string("body", html);
  match engine.render_compiled(&html_t, &context) {
    Ok(o) => Ok(o),
    Err(o) => {
      log::error!("md2html:{}", o);
      Err(crate::error::AppError::Template(o))
    }
  }
}
