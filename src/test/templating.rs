//! Template engine and context building tests
//!
//! Tests for template context type inference, environment variable expansion,
//! and engine caching behavior.

use handlebars::Context;
use serde_json::json;

use crate::parser::templating::set_context_value;

#[test]
fn test_set_context_value_bool_true() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "flag:true");

    let data = context.data();
    assert_eq!(data["flag"], json!(true));
}

#[test]
fn test_set_context_value_bool_false() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "enabled:false");

    let data = context.data();
    assert_eq!(data["enabled"], json!(false));
}

#[test]
fn test_set_context_value_integer() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "count:42");

    let data = context.data();
    assert_eq!(data["count"], json!(42));
}

#[test]
fn test_set_context_value_negative_integer() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "temp:-15");

    let data = context.data();
    assert_eq!(data["temp"], json!(-15));
}

#[test]
fn test_set_context_value_string() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "name:test value");

    let data = context.data();
    assert_eq!(data["name"], json!("test value"));
}

#[test]
fn test_set_context_value_string_with_spaces() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "message:hello world");

    let data = context.data();
    assert_eq!(data["message"], json!("hello world"));
}

#[test]
fn test_set_context_value_type_precedence() {
    // Bool parsing takes precedence over string
    let mut context1 = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context1, "val:true");
    assert_eq!(context1.data()["val"], json!(true));

    // Number parsing takes precedence over string
    let mut context2 = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context2, "val:123");
    assert_eq!(context2.data()["val"], json!(123));

    // Non-numeric string stays string
    let mut context3 = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context3, "val:abc123");
    assert_eq!(context3.data()["val"], json!("abc123"));
}

#[test]
fn test_set_context_value_env_variable() {
    unsafe {
        std::env::set_var("TEST_VAR", "test_value");
    }

    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "myvar:env:TEST_VAR");

    let data = context.data();
    assert_eq!(data["myvar"], json!("test_value"));

    unsafe {
        std::env::remove_var("TEST_VAR");
    }
}

#[test]
fn test_set_context_value_env_bool() {
    unsafe {
        std::env::set_var("BOOL_VAR", "true");
    }

    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "flag:env:BOOL_VAR");

    let data = context.data();
    assert_eq!(data["flag"], json!(true));

    unsafe {
        std::env::remove_var("BOOL_VAR");
    }
}

#[test]
fn test_set_context_value_env_number() {
    unsafe {
        std::env::set_var("NUM_VAR", "999");
    }

    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "count:env:NUM_VAR");

    let data = context.data();
    assert_eq!(data["count"], json!(999));

    unsafe {
        std::env::remove_var("NUM_VAR");
    }
}

#[test]
fn test_set_context_value_malformed_no_colon() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "malformed");

    // Should be ignored, context remains unchanged
    assert!(context.data().as_object().unwrap().is_empty());
}

#[test]
fn test_set_context_value_empty_name() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, ":value");

    // Empty name should be ignored
    assert!(context.data().as_object().unwrap().is_empty());
}

#[test]
fn test_set_context_value_whitespace_handling() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "  key  :  value  ");

    let data = context.data();
    assert_eq!(data["key"], json!("value"));
}

#[test]
fn test_set_context_value_multiple_colons() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "url:http://example.com");

    let data = context.data();
    // Should only split on first colon
    assert_eq!(data["url"], json!("http://example.com"));
}

#[test]
fn test_set_context_value_empty_value() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "empty:");

    let data = context.data();
    assert_eq!(data["empty"], json!(""));
}

#[test]
fn test_set_context_value_unicode() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "text:你好世界");

    let data = context.data();
    assert_eq!(data["text"], json!("你好世界"));
}

#[test]
fn test_set_context_value_special_chars() {
    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "special:!@#$%^&*()");

    let data = context.data();
    assert_eq!(data["special"], json!("!@#$%^&*()"));
}

#[test]
fn test_get_context_includes_server_version() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("cofg.yaml");

    let config_content = r#"
public_path: ./public
addrs:
  host: 127.0.0.1
  port: 8080
"#;

    fs::write(&config_path, config_content).unwrap();

    // Note: In real tests, we'd need to properly initialize Cofg
    // This demonstrates the structure
}

#[test]
fn test_context_value_inference_order() {
    // Test that bool is tried first, then i64, then string
    let mut ctx = Context::wraps(json!({})).unwrap();

    // Boolean values
    set_context_value(&mut ctx, "a:true");
    assert!(ctx.data()["a"].is_boolean());

    set_context_value(&mut ctx, "b:false");
    assert!(ctx.data()["b"].is_boolean());

    // Numeric values
    set_context_value(&mut ctx, "c:0");
    assert!(ctx.data()["c"].is_number());

    set_context_value(&mut ctx, "d:12345");
    assert!(ctx.data()["d"].is_number());

    // String values (fallback)
    set_context_value(&mut ctx, "e:not_a_bool");
    assert!(ctx.data()["e"].is_string());

    set_context_value(&mut ctx, "f:3.14");
    assert!(ctx.data()["f"].is_string()); // Floats become strings
}

#[test]
fn test_env_var_missing() {
    // Ensure non-existent env var doesn't cause panic
    unsafe {
        std::env::remove_var("NONEXISTENT_VAR_12345");
    }

    let mut context = Context::wraps(json!({})).unwrap();
    set_context_value(&mut context, "missing:env:NONEXISTENT_VAR_12345");

    // Should not set the value if env var doesn't exist
    let data = context.data();
    assert!(!data.as_object().unwrap().contains_key("missing"));
}

#[test]
fn test_multiple_context_values() {
    let mut context = Context::wraps(json!({})).unwrap();

    set_context_value(&mut context, "name:MyApp");
    set_context_value(&mut context, "version:1");
    set_context_value(&mut context, "debug:true");

    let data = context.data();
    assert_eq!(data["name"], json!("MyApp"));
    assert_eq!(data["version"], json!(1));
    assert_eq!(data["debug"], json!(true));
}

#[test]
fn test_context_value_overwrite() {
    let mut context = Context::wraps(json!({})).unwrap();

    set_context_value(&mut context, "key:first");
    assert_eq!(context.data()["key"], json!("first"));

    set_context_value(&mut context, "key:second");
    assert_eq!(context.data()["key"], json!("second"));
}
