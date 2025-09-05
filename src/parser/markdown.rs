//! markdown

use std::{ fs::{ read_to_string, remove_file, write }, path::Path };

use wax::Glob;

use crate::parser::md2html;

pub(crate) fn md2html_all() -> Result<(), Box<dyn std::error::Error>> {
  let md_files = Glob::new("**/*.{md,markdown}")?;

  for entry in md_files.walk("./public/") {
    let entry = entry?;
    let path = entry.path().to_path_buf();
    let out_path_obj = path.with_extension("html");

    write(&out_path_obj, md2html(read_to_string(&path)?)?)?;
  }
  Ok(())
}

pub(crate) fn make_toc() -> Result<(), Box<dyn std::error::Error>> {
  let c = &crate::cofg::Cofg::new();

  let out_path = &Path::new("./public/").join(std::ops::Deref::deref(&c.toc.path));
  let out_dir = &out_path.parent().unwrap().canonicalize().unwrap();

  if out_path.exists() {
    remove_file(out_path)?;
  }
  let mut toc_str = String::from("# toc\n\n");
  for entry in Glob::new(&format!("**/*.{{{}}}", c.toc.ext.join(",")))?.walk(".\\public\\") {
    let entry = entry?;
    let path = entry.path().canonicalize().unwrap().strip_prefix(out_dir).unwrap().to_path_buf();

    toc_str += format!(
      "- [{}]({})\n",
      path.with_extension("").display(),
      percent_encoding::utf8_percent_encode(
        &path.display().to_string(),
        percent_encoding::NON_ALPHANUMERIC
      )
    ).as_str();
  }
  write(out_path, md2html(toc_str)?)?;

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
