//! parser

use crate::parser::templating::set_context_value;

pub(crate) mod markdown;
pub(crate) mod templating;

/// input md str
/// return html str
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
