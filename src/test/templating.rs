//! tests for templating context parsing

use handlebars::Context;

use crate::parser::templating::{ get_context, set_context_value };
use crate::{ cofg::config::Cofg, parser::md2html };

#[test]
fn set_context_value_parses_types_and_env() {
  let mut ctx = Context::null();
  // bool
  set_context_value(&mut ctx, "flag:true");
  // number
  set_context_value(&mut ctx, "retries:3");
  // string
  set_context_value(&mut ctx, "mode:fast");

  let data = ctx.data();
  assert_eq!(data["flag"], true);
  assert_eq!(data["retries"], 3);
  assert_eq!(data["mode"], "fast");
  assert!(data.get("answer").is_none());
}

#[test]
fn get_context_includes_server_version_and_template_values() {
  let mut c = Cofg::default();
  c.templating.value = Some(vec!["site:true".into(), "count:7".into(), "name:demo".into()]);
  let ctx = get_context(&c);
  let data = ctx.data();
  assert!(data["server-version"].is_string());
  assert_eq!(data["site"], true);
  assert_eq!(data["count"], 7);
  assert_eq!(data["name"], "demo");
}

#[test]
fn md2html_injects_body_and_title() {
  let c = Cofg::default();
  // minimal markdown
  let html = md2html("# Title".to_string(), &c, vec!["path:demo.md".into()]).unwrap();
  assert!(html.contains("<h1>Title</h1>"));
  assert!(html.contains("<title>demo.md</title>"));
}
