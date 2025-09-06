//! parser

pub(crate) mod markdown;
pub(crate) mod templating;

/// input md str
/// return html str
pub(crate) fn md2html(md: String, c: &crate::cofg::Cofg) -> crate::error::AppResult<String> {
  // let c = &crate::cofg::Cofg::new();
  let mut engine = templating::get_engine(c);
  let mut context = templating::get_context(c);

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
