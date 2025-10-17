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

#[test]
fn md2html_cache_same_file_same_output() {
  let c = Cofg::default();
  // Use an existing small markdown file in test data
  let rel = "dir/test.md";
  let abs = std::path::Path::new(&c.public_path).join(rel);
  let content = std::fs::read_to_string(&abs).unwrap();
  let a = md2html(content.clone(), &c, vec![format!("path:{}", rel)]).unwrap();
  let b = md2html(content, &c, vec![format!("path:{}", rel)]).unwrap();
  assert_eq!(a, b);
}

#[test]
fn md2html_cache_invalidates_on_change() {
  use std::io::Write;
  let c = Cofg::default();
  let rel = "dir/test-cache.md";
  let abs = std::path::Path::new(&c.public_path).join(rel);

  // create temporary file under public
  std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
  let mut f = std::fs::File::create(&abs).unwrap();
  writeln!(f, "# A").unwrap();
  f.sync_all().unwrap();

  let first = md2html("# A".into(), &c, vec![format!("path:{}", rel)]).unwrap();

  // Modify the file content and its mtime
  std::thread::sleep(std::time::Duration::from_millis(1100));
  std::fs::write(&abs, "# B\n").unwrap();

  let second = md2html("# B".into(), &c, vec![format!("path:{}", rel)]).unwrap();
  assert_ne!(first, second);

  // cleanup
  let _ = std::fs::remove_file(&abs);
}

#[test]
fn toc_is_memoized_for_root() {
  let c = Cofg::default();
  let root = std::path::Path::new(&c.public_path);
  let t1 = get_toc(root, &c, Some("index".to_string())).unwrap();
  let t2 = get_toc(root, &c, Some("index".to_string())).unwrap();
  assert_eq!(t1, t2);
}
