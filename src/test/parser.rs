//! tests for parser::md2html

#[test]
fn md2html_minimal_h1() {
  use crate::{ cofg, parser };
  let _g = super::common::meta_mutex().lock().unwrap();

  // ensure a minimal template exists under ./meta for this test
  std::fs::create_dir_all("./meta").expect("mkdir meta");
  std::fs
    ::write(
      "./meta/html-t.templating",
      "<html><body>{{ body }} -- v{{ server-version }}</body></html>"
    )
    .expect("write template");

  let mut cfg = cofg::Cofg::default();
  cfg.templating.hot_reload = true; // ensure engine reloads template

  let html = parser::md2html("# h1".to_string(), &cfg, vec![]).expect("md2html error");
  assert!(html.contains("&lt;h1&gt;h1&lt;/h1&gt;"), "should contain escaped rendered h1: {html}");
  assert!(html.contains(&format!("v{}", env!("CARGO_PKG_VERSION"))), "has version");

  let _ = std::fs::remove_dir_all("./meta");
}
