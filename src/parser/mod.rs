//! parser
pub(crate) mod markdown;
pub(crate) mod templating;

pub(crate) fn md2html(
  input_path: &std::path::PathBuf,
  output_path: &std::path::PathBuf
) -> Result<(), Box<dyn std::error::Error>> {
  let c = &crate::cofg::Cofg::new();
  let mut engine = templating::get_engine(c);
  let mut context = templating::get_context(c);

  let html_t = engine.compile_to_bytecode("html-t.templating")?;

  let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
  let ast = markdown::parser_md(input);
  log::trace!("ast={ast:#?}");
  let html = &markdown_ppp::html_printer::render_html(
    &ast,
    markdown_ppp::html_printer::config::Config::default()
  );

  context.set_string("body", html);
  match engine.render_compiled(&html_t, &context) {
    Ok(o) => {
      std::fs::write(output_path, o).unwrap();
      Ok(())
    }
    Err(o) => {
      log::error!("md2html:{}:{}", input_path.display(), o);
      Err(o.into())
    }
  }
}
