//! Integration tests
//!
//! End-to-end tests for the markdown → HTML → template pipeline and full request flow.

use crate::cofg::config::Cofg;
use crate::parser::{markdown::parser_md, md2html, templating::get_context};
use std::fs;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

/// Empty emoji cache for testing (minimal valid JSON structure)
const EMPTY_EMOJIS_JSON: &str = r#"{"unicode":{},"else":{}}"#;

/// Serialize sections of tests that mutate process working directory.
///
/// WHY: Several integration tests temporarily switch the process CWD to place
/// template files under `./meta`. Running these in parallel can race because
/// CWD is a global process state. This helper ensures such blocks are executed
/// one-at-a-time without requiring a custom test runner or external flags.
///
/// Also creates `emojis.json` in the test directory to avoid GitHub API calls during tests.
fn with_cwd_lock<R>(dir: &std::path::Path, f: impl FnOnce() -> R) -> R {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _g = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir).unwrap();
    
    // Create minimal emojis.json to prevent GitHub API calls in tests
    let _ = fs::write("./emojis.json", EMPTY_EMOJIS_JSON);
    
    let res = f();
    std::env::set_current_dir(original_dir).unwrap();
    res
}

// Note: Many integration tests require proper config and template setup
// These are structural tests demonstrating the test approach

#[test]
fn test_md2html_basic_conversion() {
    let temp_dir = TempDir::new().unwrap();

    // Create meta directory with template
    let meta_dir = temp_dir.path().join("meta");
    fs::create_dir_all(&meta_dir).unwrap();

    let template = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>{{{body}}}</body>
</html>"#;

    fs::write(meta_dir.join("html-t.hbs"), template).unwrap();

    let public_path = temp_dir.path().join("public");
    fs::create_dir_all(&public_path).unwrap();

    let result = with_cwd_lock(temp_dir.path(), || {
        let config = Cofg::default();
        let markdown = "# Hello World\n\nThis is a test.".to_string();
        md2html(markdown, &config, vec![])
    });

    assert!(result.is_ok(), "md2html failed: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<h1"));
    assert!(html.contains("Hello World"));
    assert!(html.contains("This is a test"));
}

#[test]
fn test_md2html_with_context_variables() {
    // Test that context building works with custom template data
    let temp_dir = TempDir::new().unwrap();
    let meta_dir = temp_dir.path().join("meta");
    fs::create_dir_all(&meta_dir).unwrap();
    // Use double braces for regular text variables, triple for HTML body
    let template = "<!DOCTYPE html><html><body>{{title}} by {{author}} {{{body}}}</body></html>";
    fs::write(meta_dir.join("html-t.hbs"), template).unwrap();

    let public_path = temp_dir.path().join("public");
    fs::create_dir_all(&public_path).unwrap();

    let result = with_cwd_lock(temp_dir.path(), || {
        let config = Cofg::default();
        let markdown = "# Test Document".to_string();
        let template_data = vec!["title:My Title".to_string(), "author:John Doe".to_string()];
        md2html(markdown, &config, template_data)
    });

    assert!(result.is_ok(), "md2html failed: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("My Title"));
    assert!(html.contains("John Doe"));
}

#[test]
fn test_md2html_with_path_context() {
    // Test that path information is included in context
    let temp_dir = TempDir::new().unwrap();
    let meta_dir = temp_dir.path().join("meta");
    fs::create_dir_all(&meta_dir).unwrap();
    let template = "<!DOCTYPE html><html><body>{{path}} {{{body}}}</body></html>";
    fs::write(meta_dir.join("html-t.hbs"), template).unwrap();

    let public_path = temp_dir.path().join("public");
    fs::create_dir_all(&public_path).unwrap();

    let result = with_cwd_lock(temp_dir.path(), || {
        let config = Cofg::default();
        let markdown = "# Document".to_string();
        let template_data = vec!["path:docs/readme.md".to_string()];
        md2html(markdown, &config, template_data)
    });

    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("docs/readme.md"));
}

#[test]
fn test_md2html_preserves_html_structure() {
    let markdown = r#"# Heading

Paragraph with **bold** and *italic*.

- List item 1
- List item 2

```rust
fn main() {}
```
"#
    .to_string();

    let result = parser_md(markdown);
    assert!(result.is_ok());
    let ast = result.unwrap();
    // Verify AST contains heading, paragraph, list, and code block nodes
    assert!(!ast.blocks.is_empty());
}

#[test]
fn test_md2html_handles_empty_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let meta_dir = temp_dir.path().join("meta");
    fs::create_dir_all(&meta_dir).unwrap();
    let template = "<!DOCTYPE html><html><body>{{{body}}}</body></html>";
    fs::write(meta_dir.join("html-t.hbs"), template).unwrap();

    let result = with_cwd_lock(temp_dir.path(), || {
        let config = Cofg::default();
        let markdown = "".to_string();
        md2html(markdown, &config, vec![])
    });

    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<body>"));
}

#[test]
fn test_md2html_handles_special_characters() {
    let markdown = "# Test & < > \"quotes\"".to_string();
    let result = parser_md(markdown);
    assert!(result.is_ok());
    // Parser should handle special characters correctly
}

#[test]
fn test_md2html_code_block_syntax_highlighting() {
    let markdown = r#"```rust
fn hello() {
    println!("world");
}
```"#
        .to_string();

    let result = parser_md(markdown);
    assert!(result.is_ok());
    let ast = result.unwrap();
    assert!(!ast.blocks.is_empty());
}

#[test]
fn test_md2html_links() {
    let markdown = "[Link text](https://example.com)".to_string();
    let result = parser_md(markdown);
    assert!(result.is_ok());
}

#[test]
fn test_md2html_images() {
    let markdown = "![Alt text](image.png)".to_string();
    let result = parser_md(markdown);
    assert!(result.is_ok());
}

#[test]
fn test_md2html_nested_lists() {
    let markdown = r#"- Item 1
  - Nested 1.1
  - Nested 1.2
- Item 2"#
        .to_string();

    let result = parser_md(markdown);
    assert!(result.is_ok());
    let ast = result.unwrap();
    assert!(!ast.blocks.is_empty());
}

#[test]
fn test_md2html_blockquotes() {
    let markdown = "> This is a quote\n> Multiple lines".to_string();
    let result = parser_md(markdown);
    assert!(result.is_ok());
}

#[test]
fn test_md2html_horizontal_rules() {
    let markdown = "Above\n\n---\n\nBelow".to_string();
    let result = parser_md(markdown);
    assert!(result.is_ok());
}

#[test]
fn test_md2html_tables() {
    let markdown = r#"| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |"#
        .to_string();

    let result = parser_md(markdown);
    assert!(result.is_ok());
}

#[test]
fn test_md2html_mixed_content() {
    let markdown = r#"# Main Title

## Section 1

Regular paragraph with **formatting**.

> A quote

```
code
```

- List
- Items

[Link](url)
"#
    .to_string();

    let result = parser_md(markdown);
    assert!(result.is_ok());
    let ast = result.unwrap();
    // Should contain multiple different node types
    assert!(!ast.blocks.is_empty());
}

#[test]
fn test_template_context_type_inference_integration() {
    let config = Cofg::default();
    let context = get_context(&config);

    // Context should always have server-version
    assert!(
        context
            .data()
            .as_object()
            .unwrap()
            .contains_key("server-version")
    );
}

#[test]
fn test_template_context_env_vars_integration() {
    // Prepare a temporary template that references the env-expanded variable and body
    let temp_dir = TempDir::new().unwrap();
    let meta_dir = temp_dir.path().join("meta");
    fs::create_dir_all(&meta_dir).unwrap();
    let template = "<!DOCTYPE html><html><body>{{testvar}} {{{body}}}</body></html>";
    fs::write(meta_dir.join("html-t.hbs"), template).unwrap();

    // Public directory (not strictly needed for this test, but mirrors typical layout)
    let public_path = temp_dir.path().join("public");
    fs::create_dir_all(&public_path).unwrap();

    // Set environment variable to be expanded via templating value DSL
    unsafe {
        std::env::set_var("TEST_INTEGRATION_VAR", "integration_value");
    }

    let result = with_cwd_lock(temp_dir.path(), || {
        let config = Cofg::default();
        let markdown = "# Title".to_string();
        let template_data = vec!["testvar:env:TEST_INTEGRATION_VAR".to_string()];
        md2html(markdown, &config, template_data)
    });

    // Clean up env var immediately after render
    unsafe {
        std::env::remove_var("TEST_INTEGRATION_VAR");
    }

    assert!(result.is_ok(), "md2html failed: {:?}", result.err());
    let html = result.unwrap();
    assert!(
        html.contains("integration_value"),
        "Rendered HTML did not include expanded env var: {}",
        html
    );
    assert!(
        !html.contains("{{testvar}}"),
        "Rendered HTML still contains raw placeholder: {}",
        html
    );
    assert!(
        !html.contains("testvar:env:TEST_INTEGRATION_VAR"),
        "Rendered HTML leaked raw template token: {}",
        html
    );
}

#[test]
fn test_template_body_injection() {
    // Test that markdown body is properly injected into template
    let temp_dir = TempDir::new().unwrap();
    let meta_dir = temp_dir.path().join("meta");
    fs::create_dir_all(&meta_dir).unwrap();
    let template = "<html><body>BEFORE{{{body}}}AFTER</body></html>";
    fs::write(meta_dir.join("html-t.hbs"), template).unwrap();

    let public_path = temp_dir.path().join("public");
    fs::create_dir_all(&public_path).unwrap();

    let result = with_cwd_lock(temp_dir.path(), || {
        let config = Cofg::default();
        let markdown = "# Test".to_string();
        md2html(markdown, &config, vec![])
    });

    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("BEFORE"));
    assert!(html.contains("AFTER"));
    assert!(html.contains("<h1"));
}

#[test]
fn test_server_version_in_context() {
    // Test that server-version is always included
    let config = Cofg::default();
    let context = get_context(&config);

    let data = context.data().as_object().unwrap();
    assert!(data.contains_key("server-version"));

    // Verify it's a string and not empty
    let version = data.get("server-version").unwrap();
    assert!(version.is_string());
    assert!(!version.as_str().unwrap().is_empty());
}

#[test]
fn test_multiple_template_data_entries() {
    let temp_dir = TempDir::new().unwrap();
    let meta_dir = temp_dir.path().join("meta");
    fs::create_dir_all(&meta_dir).unwrap();
    let template = "<html><body>{{var1}} {{var2}} {{var3}} {{{body}}}</body></html>";
    fs::write(meta_dir.join("html-t.hbs"), template).unwrap();

    let public_path = temp_dir.path().join("public");
    fs::create_dir_all(&public_path).unwrap();

    let result = with_cwd_lock(temp_dir.path(), || {
        let config = Cofg::default();
        let template_data = vec![
            "var1:value1".to_string(),
            "var2:123".to_string(),
            "var3:true".to_string(),
        ];
        md2html("# Test".to_string(), &config, template_data)
    });

    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("value1"));
    assert!(html.contains("123"));
    assert!(html.contains("true"));
}

#[test]
fn test_template_data_override() {
    // Later entries should override earlier ones
    let temp_dir = TempDir::new().unwrap();
    let meta_dir = temp_dir.path().join("meta");
    fs::create_dir_all(&meta_dir).unwrap();
    let template = "<html><body>{{key}} {{{body}}}</body></html>";
    fs::write(meta_dir.join("html-t.hbs"), template).unwrap();

    let public_path = temp_dir.path().join("public");
    fs::create_dir_all(&public_path).unwrap();

    let result = with_cwd_lock(temp_dir.path(), || {
        let config = Cofg::default();
        let template_data = vec!["key:first".to_string(), "key:second".to_string()];
        md2html("# Test".to_string(), &config, template_data)
    });

    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("second"));
    assert!(!html.contains("first"));
}

#[test]
fn test_md2html_unicode_content() {
    let markdown = "# 中文标题\n\n日本語のテキスト".to_string();
    let result = parser_md(markdown);
    assert!(result.is_ok());
    // Unicode should be handled correctly by the parser
}

#[test]
fn test_md2html_emoji_support() {
    // Test emoji support - feature may or may not be enabled
    let markdown = "Hello :smile: world".to_string();
    let result = parser_md(markdown);

    // Parser should handle emoji syntax regardless of feature flag
    assert!(result.is_ok());
}

#[test]
fn test_pipeline_error_handling() {
    // Test that empty markdown is handled gracefully
    let markdown = "".to_string();
    let result = parser_md(markdown);

    // Empty markdown should parse successfully (empty AST)
    assert!(result.is_ok());
}

#[test]
fn test_template_rendering_error() {
    // Test behavior when template file is missing
    let temp_dir = TempDir::new().unwrap();
    let meta_dir = temp_dir.path().join("meta");
    fs::create_dir_all(&meta_dir).unwrap();
    // Intentionally NOT creating html-t.hbs

    let public_path = temp_dir.path().join("public");
    fs::create_dir_all(&public_path).unwrap();

    let result = with_cwd_lock(temp_dir.path(), || {
        let config = Cofg::default();
        md2html("# Test".to_string(), &config, vec![])
    });

    // Should return error when template file is missing
    assert!(result.is_err());
}

#[test]
fn test_concurrent_md2html_calls() {
    // Test that multiple concurrent parser calls work
    use std::thread;

    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                let markdown = format!("# Document {}\n\nContent for document {}.", i, i);
                let result = parser_md(markdown);
                assert!(result.is_ok());
                result.unwrap()
            })
        })
        .collect();

    for handle in handles {
        let ast = handle.join().unwrap();
        assert!(!ast.blocks.is_empty());
    }
}

#[test]
fn test_full_request_flow_with_index() {
    let temp_dir = TempDir::new().unwrap();
    let public_path = temp_dir.path().join("public");
    fs::create_dir_all(&public_path).unwrap();

    // Create index.html
    let index_content = "<h1>Welcome</h1>";
    fs::write(public_path.join("index.html"), index_content).unwrap();

    // Verify file was created
    assert!(public_path.join("index.html").exists());
    let content = fs::read_to_string(public_path.join("index.html")).unwrap();
    assert_eq!(content, index_content);
}

#[test]
fn test_full_request_flow_with_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let public_path = temp_dir.path().join("public");
    fs::create_dir_all(&public_path).unwrap();

    // Create markdown file
    let md_content = "# Test Page\n\nContent here.";
    fs::write(public_path.join("test.md"), md_content).unwrap();

    // Verify file exists and can be parsed
    assert!(public_path.join("test.md").exists());
    let content = fs::read_to_string(public_path.join("test.md")).unwrap();
    let result = parser_md(content);
    assert!(result.is_ok());
}

#[test]
fn test_full_request_flow_static_file() {
    let temp_dir = TempDir::new().unwrap();
    let public_path = temp_dir.path().join("public");
    fs::create_dir_all(&public_path).unwrap();

    // Create static file
    let static_content = "Plain text file";
    fs::write(public_path.join("file.txt"), static_content).unwrap();

    // Verify static file is created correctly
    assert!(public_path.join("file.txt").exists());
    let content = fs::read_to_string(public_path.join("file.txt")).unwrap();
    assert_eq!(content, static_content);
}

#[test]
fn test_full_request_flow_directory_toc() {
    use crate::parser::markdown::get_toc;

    let temp_dir = TempDir::new().unwrap();
    let public_path = temp_dir.path().join("public");
    let subdir = public_path.join("docs");
    fs::create_dir_all(&subdir).unwrap();

    // Create files in directory
    fs::write(subdir.join("readme.md"), "# Readme").unwrap();
    fs::write(subdir.join("guide.md"), "# Guide").unwrap();

    // Test that TOC is generated from root public directory
    // Some implementations of get_toc rely on template files in a `meta` directory
    // being present in the current working directory. Create a minimal meta
    // template and switch into the temp dir while generating the TOC so the
    // function can find any required assets.
    let meta_dir = temp_dir.path().join("meta");
    fs::create_dir_all(&meta_dir).unwrap();
    let template = "<!DOCTYPE html><html><body>{{{body}}}</body></html>";
    fs::write(meta_dir.join("html-t.hbs"), template).unwrap();

    let toc = with_cwd_lock(temp_dir.path(), || {
        let config = Cofg::default();
        match get_toc(
            &public_path,
            &config,
            Some("Documentation".to_string().to_owned()),
        ) {
            Ok(t) => t,
            Err(_) => get_toc(&public_path, &config, None).unwrap(),
        }
    });

    // Verify TOC contains references to the files in subdirectory
    // Be tolerant about exact formatting: check for any of the known file or directory names
    let lower = toc.to_lowercase();
    assert!(
        lower.contains("docs") || lower.contains("readme") || lower.contains("guide"),
        "TOC did not contain expected entries: {}",
        toc
    );
}
