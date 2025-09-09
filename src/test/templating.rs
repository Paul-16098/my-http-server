//! tests for templating context parsing

#[test]
fn templating_context_parses_basic_types() {
  use crate::{ cofg, parser::templating };

  // ensure meta folder and a small test template exist
  std::fs::create_dir_all("./meta").expect("create meta");
  std::fs
    ::write("./meta/inline.test.templating", "{{ a }} {{ b }} {{ c }} {{ server-version }}")
    .expect("write inline template");

  // build a config with templating values
  let mut cfg = cofg::Cofg::default();
  cfg.templating.value = Some(vec!["a:true".into(), "b:1".into(), "c:txt".into()]);
  cfg.templating.hot_reload = true; // ensure engine sees freshly written templates

  let ctx = templating::get_context(&cfg);
  let mut engine = templating::get_engine(&cfg);
  let t = engine.compile_to_bytecode("inline.test.templating").expect("compile template file");
  let out = engine.render_compiled(&t, &ctx).expect("render should succeed");

  assert!(out.contains("true"), "bool should render as true: {out}");
  assert!(out.contains("1"), "number should render as 1: {out}");
  assert!(out.contains("txt"), "string should render: {out}");
  assert!(out.contains(env!("CARGO_PKG_VERSION")), "server-version injected");

  std::fs::remove_dir_all("./meta").unwrap();
}
