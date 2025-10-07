//! tests for templating context parsing

#[test]
fn templating_context_parses_basic_types() {
  use crate::{ cofg::config, parser::templating };
  let _g = super::common::meta_mutex().lock().unwrap();

  // ensure meta folder and a small test template exist
  std::fs::create_dir_all("./meta").expect("create meta");
  std::fs
    ::write("./meta/inline.test.templating", "{{ a }} {{ b }} {{ c }} {{ server-version }}")
    .expect("write inline template");

  // build a config with templating values
  let mut cfg = config::Cofg::default();
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

  // best-effort cleanup; ignore errors in parallel runs
  let _ = std::fs::remove_dir_all("./meta");
}

#[test]
fn templating_context_handles_quotes_colon_bool_synonyms() {
  use crate::{ cofg::config, parser::templating };
  let _g = super::common::meta_mutex().lock().unwrap();

  // prepare template dir
  std::fs::create_dir_all("./meta").expect("create meta");
  std::fs
    ::write(
      "./meta/inline2.test.templating",
      "{{ title }} :: {{ mode }} :: {{ app }} :: v{{ server-version }}"
    )
    .expect("write inline template2");

  let mut cfg = config::Cofg::default();
  cfg.templating.hot_reload = true;
  cfg.templating.value = Some(
    vec!["title:\"Hello: World\"".into(), "mode:true".into(), "app:My App".into()]
  );

  let ctx = templating::get_context(&cfg);
  let mut engine = templating::get_engine(&cfg);
  let t = engine.compile_to_bytecode("inline2.test.templating").expect("compile");
  let out = engine.render_compiled(&t, &ctx).expect("render ok");

  assert!(out.contains("Hello: World"), "quoted value with colon kept: {out}");
  assert!(out.contains("true"), "on => true: {out}");
  assert!(out.contains("My App"), "string value rendered: {out}");
  assert!(out.contains(env!("CARGO_PKG_VERSION")), "server-version injected");

  // best-effort cleanup
  let _ = std::fs::remove_dir_all("./meta");
}
