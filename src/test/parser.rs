//! Parser tests - Markdown parsing and templating logic
//!
//! WHY: Validate core rendering pipeline:
//! - Markdown AST parsing works correctly
//! - Template context assembly and type inference
//! - md2html integration (markdown → HTML → template)
//! - TOC generation logic

use markdown_ppp::ast::{
    Block, CodeBlock,
    CodeBlockKind::Fenced,
    Document, Heading,
    HeadingKind::Atx,
    Inline::Text,
    List,
    ListBulletKind::Dash,
    ListItem,
    ListKind::{Bullet, Ordered},
    ListOrderedKindOptions,
};
use simple_test_case::test_case;

use crate::cofg::config::Cofg;
use crate::parser::{markdown, md2html, templating};
use crate::test::config::create_test_dir;
use std::fs;

#[test_case(
    "# Hello World\n\nThis is a test.",
    Document {
        blocks: vec![
            Block::Heading(Heading {
                kind: Atx(1),
                content: [Text("Hello World".to_string())].to_vec()
            }),
            Block::Paragraph([Text("This is a test.".to_string())].to_vec())
        ]
    }
    ; "Basic markdown with heading and paragraph"
)]
#[test_case("", Document { blocks: vec![] }; "Empty markdown")]
#[test_case(
    r#"
# Code Example

```rust
fn main() {
    println!("Hello, world!");
}
```
"#,
    Document {
        blocks: vec![Block::Heading(Heading {
            kind: Atx(1),
            content: [Text("Code Example".to_string())].to_vec()
        }), Block::CodeBlock(CodeBlock {
            kind: Fenced {
                info: Some("rust".to_string())
            },
            literal: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string()
        })]
    }
    ; "Markdown with code block"
)]
#[test_case(
    r#"
# Shopping List

- Item 1
- Item 2
- Item 3

1. First
2. Second
3. Third
"#,
    Document {
        blocks: vec![
            Block::Heading(Heading {
                kind: Atx(1),
                content: [Text("Shopping List".to_string())].to_vec()
            }),
            Block::List(List {
                kind: Bullet(Dash),
                items: vec![
                    ListItem {
                        task: None,
                        blocks: vec![Block::Paragraph([Text("Item 1".to_string())].to_vec())]
                    },
                    ListItem {
                        task: None,
                        blocks: vec![Block::Paragraph([Text("Item 2".to_string())].to_vec())]
                    },
                    ListItem {
                        task: None,
                        blocks: vec![Block::Paragraph([Text("Item 3".to_string())].to_vec())]
                    }
                ]
            }),
            Block::List(List {
                kind: Ordered(ListOrderedKindOptions { start: 1 }),
                items: vec![
                    ListItem {
                        task: None,
                        blocks: vec![Block::Paragraph([Text("First".to_string())].to_vec())]
                    },
                    ListItem {
                        task: None,
                        blocks: vec![Block::Paragraph([Text("Second".to_string())].to_vec())]
                    },
                    ListItem {
                        task: None,
                        blocks: vec![Block::Paragraph([Text("Third".to_string())].to_vec())]
                    }
                ]
            })
        ]
    }
    ; "Markdown with lists"
)]
#[actix_web::test]
async fn test_markdown_parsing(md: &'static str, expected_ast: Document) {
    let md = md.to_string();

    let result = markdown::parser_md(md);
    assert!(result.is_ok(), "Basic markdown should parse successfully");

    let ast = result.unwrap();
    assert_eq!(ast, expected_ast)
}

#[actix_web::test]
async fn test_context_creation() {
    let config = Cofg::default();
    let context = templating::get_context(&config);

    // Context should contain server-version
    assert!(
        context.data().get("server-version").is_some(),
        "Context should have server-version"
    );
}

#[actix_web::test]
async fn test_set_context_value_string() {
    let config = Cofg::default();
    let mut context = templating::get_context(&config);

    templating::set_context_value(&mut context, "title:My Page");

    let data = context.data();
    assert_eq!(
        data.get("title").and_then(|v| v.as_str()),
        Some("My Page"),
        "String value should be set correctly"
    );
}

#[actix_web::test]
async fn test_set_context_value_bool() {
    let config = Cofg::default();
    let mut context = templating::get_context(&config);

    templating::set_context_value(&mut context, "is_active:true");
    templating::set_context_value(&mut context, "is_disabled:false");

    let data = context.data();
    assert_eq!(
        data.get("is_active").and_then(|v| v.as_bool()),
        Some(true),
        "Boolean true should be set correctly"
    );
    assert_eq!(
        data.get("is_disabled").and_then(|v| v.as_bool()),
        Some(false),
        "Boolean false should be set correctly"
    );
}

#[actix_web::test]
async fn test_set_context_value_number() {
    let config = Cofg::default();
    let mut context = templating::get_context(&config);

    templating::set_context_value(&mut context, "count:42");
    templating::set_context_value(&mut context, "negative:-10");

    let data = context.data();
    assert_eq!(
        data.get("count").and_then(|v| v.as_i64()),
        Some(42),
        "Positive integer should be set correctly"
    );
    assert_eq!(
        data.get("negative").and_then(|v| v.as_i64()),
        Some(-10),
        "Negative integer should be set correctly"
    );
}

#[actix_web::test]
async fn test_set_context_value_invalid_format() {
    let config = Cofg::default();
    let mut context = templating::get_context(&config);

    // Invalid format (no colon) should be ignored silently
    let initial_keys: Vec<String> = context
        .data()
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();

    templating::set_context_value(&mut context, "invalid_no_colon");

    let new_keys: Vec<String> = context
        .data()
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();

    // Context should remain unchanged for invalid input
    assert_eq!(
        initial_keys.len(),
        new_keys.len(),
        "Invalid format should not add entries"
    );
}

#[actix_web::test]
async fn test_md2html_basic() {
    let temp_dir = create_test_dir();
    let template_path = temp_dir.path().join("test-template.hbs");

    // Create a minimal template
    fs::write(
        &template_path,
        "<!DOCTYPE html><html><body>{{{body}}}</body></html>",
    )
    .expect("Should write template");

    // WHY: field_reassign_with_default warning suppressed here.
    // Using struct update syntax would be extremely verbose due to Cofg's deeply nested structure
    // with 10+ nested structs (addrs, tls, middleware.logger, middleware.http_base_authentication, etc.)
    // Current pattern is more readable and maintainable for test fixtures.
    #[allow(clippy::field_reassign_with_default)]
    let config = {
        let mut c = Cofg::default();
        c.hbs_path = template_path.to_string_lossy().to_string();
        c.templating.hot_reload = false;
        c
    };

    let md = "# Test\n\nHello world!".to_string();
    let result = md2html(md, &config, vec![]);

    if let Err(e) = &result {
        eprintln!("md2html error: {:?}", e);
    }

    assert!(
        result.is_ok(),
        "md2html should succeed with valid markdown: {:?}",
        result.err()
    );
    let html = result.unwrap();
    assert_eq!(
        html,
        "<!DOCTYPE html><html><body><h1>Test</h1><p>Hello world!</p></body></html>",
    );
}

#[actix_web::test]
async fn test_md2html_with_context() {
    let temp_dir = create_test_dir();
    let template_path = temp_dir.path().join("test-template-ctx.hbs");

    // Template that uses context variable
    fs::write(
        &template_path,
        "<!DOCTYPE html><html><head><title>{{title}}</title></head><body>{{{body}}}</body></html>",
    )
    .expect("Should write template");

    // WHY: field_reassign_with_default warning suppressed here.
    // Using struct update syntax would be extremely verbose due to Cofg's deeply nested structure
    // with 10+ nested structs (addrs, tls, middleware.logger, middleware.http_base_authentication, etc.)
    // Current pattern is more readable and maintainable for test fixtures.
    #[allow(clippy::field_reassign_with_default)]
    let config = {
        let mut c = Cofg::default();
        c.hbs_path = template_path.to_string_lossy().to_string();
        c.templating.hot_reload = false;
        c
    };

    let md = "# Content".to_string();
    let context_vars = vec!["title:Test Page".to_string()];
    let result = md2html(md, &config, context_vars);

    assert!(result.is_ok(), "md2html should succeed with context vars");
    let html = result.unwrap();
    assert!(
        html.contains("<title>Test Page</title>"),
        "Template should use context variable"
    );
}

#[actix_web::test]
async fn test_toc_generation_empty_dir() {
    let temp_dir = create_test_dir();
    // WHY: field_reassign_with_default warning suppressed here.
    // Using struct update syntax would be extremely verbose due to Cofg's deeply nested structure
    // with 10+ nested structs (addrs, tls, middleware.logger, middleware.http_base_authentication, etc.)
    // Current pattern is more readable and maintainable for test fixtures.
    #[allow(clippy::field_reassign_with_default)]
    let config = {
        let mut c = Cofg::default();
        c.public_path = temp_dir.path().to_string_lossy().to_string();
        c
    };

    // TOC generation on empty directory
    let result = markdown::get_toc(temp_dir.path(), &config, Some("Test TOC".to_string()));

    if let Err(e) = &result {
        eprintln!("TOC error: {:?}", e);
    }

    assert!(
        result.is_ok(),
        "TOC generation should succeed on empty dir: {:?}",
        result.err()
    );
    let toc = result.unwrap();
    assert!(toc.contains("Test TOC"), "TOC should include title");
}

#[actix_web::test]
async fn test_toc_generation_with_files() {
    let temp_dir = create_test_dir();

    // Create some test files
    fs::write(temp_dir.path().join("test1.md"), "# Test 1").expect("Should write test1.md");
    fs::write(temp_dir.path().join("test2.html"), "<h1>Test 2</h1>")
        .expect("Should write test2.html");
    fs::write(temp_dir.path().join("readme.txt"), "README").expect("Should write readme.txt");

    // WHY: field_reassign_with_default warning suppressed here.
    // Using struct update syntax would be extremely verbose due to Cofg's deeply nested structure
    // with 10+ nested structs (addrs, tls, middleware.logger, middleware.http_base_authentication, etc.)
    // Current pattern is more readable and maintainable for test fixtures.
    #[allow(clippy::field_reassign_with_default)]
    let config = {
        let mut c = Cofg::default();
        c.public_path = temp_dir.path().to_string_lossy().to_string();
        c
    };

    let result = markdown::get_toc(temp_dir.path(), &config, Some("Files".to_string()));

    if let Err(e) = &result {
        eprintln!("TOC error: {:?}", e);
    }

    assert!(
        result.is_ok(),
        "TOC generation should succeed with files: {:?}",
        result.err()
    );
    let toc = result.unwrap();

    // Check that files with recognized extensions are included
    assert!(
        toc.contains("test1.md") || toc.contains("["),
        "TOC should reference markdown files"
    );
}

#[actix_web::test]
async fn test_markdown_with_links() {
    let md = r#"
# Links

[Google](https://www.google.com)
[Internal Link](./page.md)
"#
    .to_string();

    let result = markdown::parser_md(md);
    assert!(
        result.is_ok(),
        "Markdown with links should parse successfully"
    );
}

#[actix_web::test]
async fn test_markdown_with_images() {
    let md = r#"
# Images

![Alt text](./image.png)
![Remote image](https://example.com/image.jpg)
"#
    .to_string();

    let result = markdown::parser_md(md);
    assert!(
        result.is_ok(),
        "Markdown with images should parse successfully"
    );
}

#[actix_web::test]
async fn test_markdown_with_tables() {
    let md = r#"
# Table

| Column 1 | Column 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |
"#
    .to_string();

    let result = markdown::parser_md(md);
    assert!(
        result.is_ok(),
        "Markdown with tables should parse successfully"
    );
}

#[actix_web::test]
async fn test_context_type_inference_precedence() {
    let config = Cofg::default();
    let mut context = templating::get_context(&config);

    // Test that boolean is recognized over string
    templating::set_context_value(&mut context, "flag:true");
    assert!(
        context
            .data()
            .get("flag")
            .and_then(|v| v.as_bool())
            .is_some(),
        "Should parse as bool"
    );

    // Test that number is recognized over string
    templating::set_context_value(&mut context, "count:123");
    assert!(
        context
            .data()
            .get("count")
            .and_then(|v| v.as_i64())
            .is_some(),
        "Should parse as number"
    );

    // Test that non-parseable remains string
    templating::set_context_value(&mut context, "text:hello world");
    assert_eq!(
        context.data().get("text").and_then(|v| v.as_str()),
        Some("hello world"),
        "Should remain as string"
    );
}

#[actix_web::test]
async fn test_empty_template_data() {
    let temp_dir = create_test_dir();
    let template_path = temp_dir.path().join("empty-ctx.hbs");

    fs::write(&template_path, "<html>{{{body}}}</html>").expect("Should write template");

    // WHY: field_reassign_with_default warning suppressed here.
    // Using struct update syntax would be extremely verbose due to Cofg's deeply nested structure
    // with 10+ nested structs (addrs, tls, middleware.logger, middleware.http_base_authentication, etc.)
    // Current pattern is more readable and maintainable for test fixtures.
    #[allow(clippy::field_reassign_with_default)]
    let config = {
        let mut c = Cofg::default();
        c.hbs_path = template_path.to_string_lossy().to_string();
        c
    };

    let md = "Test".to_string();
    let result = md2html(md, &config, vec![]);

    assert!(
        result.is_ok(),
        "md2html should work with empty context vars"
    );
}
