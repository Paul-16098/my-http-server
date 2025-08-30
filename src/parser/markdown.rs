//! markdown

use crate::parser::templating::{ get_context, get_engine };

use std::fs::{ read_to_string, write };
use std::path::PathBuf;
use log::{ error, trace };
use wax::Glob;

use markdown_ppp::parser::{ parse_markdown };
use markdown_ppp::parser::config;
use markdown_ppp::html_printer::{ render_html };

pub(crate) fn md2html(c: &config::MarkdownParserConfig) -> Result<(), Box<dyn std::error::Error>> {
  let md_files = Glob::new("**/*.{md,markdown}")?;
  let mut engine = get_engine();
  let context = get_context();
  let html_t = engine.compile_to_bytecode("html-t.templating")?;
  let mut index_file: Option<PathBuf> = None;
  let mut path_lists: Vec<PathBuf> = vec![];

  for entry in md_files.walk("./public/") {
    let mut context = context.clone();

    let entry = entry?;
    let path = entry.path().to_path_buf();
    let out_path_obj = path.with_extension("html");
    let out_path = out_path_obj.to_str().unwrap();

    let input = read_to_string(&path)?;

    if input.starts_with("<!-- TOC -->") {
      index_file = Some(path);
      continue;
    } else {
      path_lists.insert(0, path.clone());
    }
    let ast = parser_md(input, c);
    trace!("ast={ast:#?}");
    let html = &render_html(&ast, markdown_ppp::html_printer::config::Config::default());
    context.set_string("body", html);
    let o = engine.render_compiled(&html_t, &context);
    if o.is_ok() {
      write(out_path, o.unwrap())?;
    } else {
      error!("md2html:{}:{}", path.display(), o.unwrap_err());
    }
  }
  if let Some(index_file) = index_file {
    let mut context = context.clone();
    make_toc(index_file.clone(), path_lists)?;
    let input = read_to_string(index_file.clone())?;

    let ast = parser_md(input, c);
    let html = &render_html(&ast, markdown_ppp::html_printer::config::Config::default());
    context.set_string("body", html);
    let o = engine.render_compiled(&html_t, &context);
    if o.is_ok() {
      write(index_file.with_extension("html"), o.unwrap())?;
    } else {
      error!("md2html:toc:{}:{}", index_file.display(), o.unwrap_err());
    }
  }
  Ok(())
}

fn make_toc(
  index_file: PathBuf,
  path_list: Vec<PathBuf>
) -> std::result::Result<(), std::io::Error> {
  let mut toc_str_list = vec!["<!-- TOC -->".to_string(), "# toc\n".to_string()];
  for path in path_list {
    let path = path.strip_prefix("./public/").unwrap();
    let path_str = path.with_extension("html").display().to_string();
    let path_str_no_ext = path.with_extension("").display().to_string();

    toc_str_list.insert(toc_str_list.len(), format!("- [{path_str_no_ext}]({path_str})"));
  }
  write(index_file, toc_str_list.join("\n") + "\n")
}
#[inline]
fn parser_md(input: String, c: &config::MarkdownParserConfig) -> markdown_ppp::ast::Document {
  // 內部需要 clone config 給 parser，但外部呼叫時可傳參考，避免重複 clone
  parse_markdown(markdown_ppp::parser::MarkdownParserState::with_config(c.clone()), &input).unwrap()
}
