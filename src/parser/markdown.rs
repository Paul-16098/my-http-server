//! markdown

use wax::Glob;

pub(crate) fn md2html_all() -> Result<(), Box<dyn std::error::Error>> {
  let md_files = Glob::new("**/*.{md,markdown}")?;

  for entry in md_files.walk("./public/") {
    let entry = entry?;
    let path = entry.path().to_path_buf();
    let out_path_obj = path.with_extension("html");

    crate::parser::md2html(&path, &out_path_obj)?;
  }
  Ok(())
}

#[inline]
pub(crate) fn parser_md(input: String) -> markdown_ppp::ast::Document {
  use markdown_ppp::parser::parse_markdown;
  // 內部需要 clone config 給 parser，但外部呼叫時可傳參考，避免重複 clone
  parse_markdown(
    markdown_ppp::parser::MarkdownParserState::with_config(
      markdown_ppp::parser::config::MarkdownParserConfig::default()
    ),
    &input
  ).unwrap()
}
