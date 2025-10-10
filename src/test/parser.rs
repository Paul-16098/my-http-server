//! tests for parser helpers

use std::path::Path;

use crate::{ cofg::config::Cofg, parser::{ markdown::get_toc, md2html } };

#[test]
fn get_toc_includes_dir_and_files_with_root_prefix() {
  let c = Cofg::default();
  let root = Path::new(&c.public_path);
  let toc = get_toc(root, &c, Some("index".to_string())).unwrap();
  assert!(toc.contains("- [test.md](/dir/test%2Emd)"));
}

#[test]
fn md2html_renders_minimal_markdown() {
  let c = Cofg::default();
  let html = md2html("# Hello".into(), &c, vec!["path:/x.md".into()]).unwrap();
  assert!(html.contains("<h1>Hello</h1>"));
}
