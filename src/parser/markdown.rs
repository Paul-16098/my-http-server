//! markdown

use std::{ fs::{ read_to_string, remove_file, write }, path::Path };

use wax::Glob;

use crate::{ error, parser::md2html };
use crate::error::{ AppResult, AppError };

pub(crate) fn md2html_all() -> AppResult<()> {
  let md_files = Glob::new("**/*.{md,markdown}")?;
  let cfg = crate::cofg::Cofg::get(false); // cached
  let public_path = cfg.public_path.clone();
  for entry in md_files.walk(public_path) {
    let entry = entry?; // WalkError already converted by ? via AppError
    let path = entry.path().to_path_buf();
    let out_path_obj = path.with_extension("html");
    write(&out_path_obj, md2html(read_to_string(&path)?, &cfg)?)?;
  }
  Ok(())
}

pub(crate) fn make_toc() -> AppResult<()> {
  let c = crate::cofg::Cofg::get(false);
  let pp = &c.public_path;

  let out_path = &Path::new(pp).join(std::ops::Deref::deref(&c.toc.path));
  let out_dir = &out_path
    .parent()
    .ok_or_else(|| AppError::Other("toc path has no parent".into()))?
    .canonicalize()?;

  if out_path.exists() {
    remove_file(out_path)?;
  }
  let mut toc_str = String::from("# toc\n\n");
  for entry in Glob::new(&format!("**/*.{{{}}}", c.toc.ext.join(",")))?.walk(pp) {
    let entry = entry?;
    let path = entry
      .path()
      .canonicalize()
      ? // io error -> AppError::Io
      .strip_prefix(out_dir)
      .map_err(|e| AppError::Other(format!("strip_prefix: {e}")))?
      .to_path_buf();

    toc_str += format!(
      "- [{}]({})\n",
      path.with_extension("").display(),
      percent_encoding::utf8_percent_encode(
        &path.display().to_string(),
        percent_encoding::NON_ALPHANUMERIC
      )
    ).as_str();
  }
  write(out_path, md2html(toc_str, &c)?)?;

  Ok(())
}

pub(crate) fn parser_md(input: String) -> error::AppResult<markdown_ppp::ast::Document> {
  use markdown_ppp::parser::parse_markdown;
  // 內部需要 clone config 給 parser，但外部呼叫時可傳參考，避免重複 clone
  Ok(
    parse_markdown(
      markdown_ppp::parser::MarkdownParserState::with_config(
        markdown_ppp::parser::config::MarkdownParserConfig::default()
      ),
      &input
    )?
  )
}
