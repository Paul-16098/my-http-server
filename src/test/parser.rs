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
use crate::test::support::create_test_dir;
use std::fs::{self, create_dir_all};

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

#[test_case("title:My Page", "title", Some("My Page") ; "String value")]
#[test_case("name:Hello", "name", Some("Hello") ; "Another string")]
#[actix_web::test]
async fn test_set_context_value_string(input: &str, key: &str, expected: Option<&str>) {
	let config = Cofg::default();
	let mut context = templating::get_context(&config);

	templating::set_context_value(&mut context, input);

	let data = context.data();
	assert_eq!(
		data.get(key).and_then(|v| v.as_str()),
		expected,
		"String value should be set correctly"
	);
}

#[test_case("is_active:true", "is_active", true ; "Boolean true")]
#[test_case("is_disabled:false", "is_disabled", false ; "Boolean false")]
#[actix_web::test]
async fn test_set_context_value_bool(input: &str, key: &str, expected: bool) {
	let config = Cofg::default();
	let mut context = templating::get_context(&config);

	templating::set_context_value(&mut context, input);

	let data = context.data();
	assert_eq!(
		data.get(key).and_then(|v| v.as_bool()),
		Some(expected),
		"Boolean value should be set correctly"
	);
}

#[test_case("count:42", "count", 42i64 ; "Positive integer")]
#[test_case("negative:-10", "negative", -10i64 ; "Negative integer")]
#[test_case("zero:0", "zero", 0i64 ; "Zero")]
#[actix_web::test]
async fn test_set_context_value_number(input: &str, key: &str, expected: i64) {
	let config = Cofg::default();
	let mut context = templating::get_context(&config);

	templating::set_context_value(&mut context, input);

	let data = context.data();
	assert_eq!(
		data.get(key).and_then(|v| v.as_i64()),
		Some(expected),
		"Number value should be set correctly"
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

// Basic markdown to HTML conversion tests
#[test_case("heading_and_text", "# Test

Hello world!", vec![] ; "Heading and text")]
#[test_case("simple_markdown", "# Welcome

Simple content", vec![] ; "Simple markdown")]
#[test_case("h2_heading", "## Section

Content here", vec![] ; "H2 heading")]
// Test with context variables
#[test_case("with_title", "# Content", vec!["title:Test Page".to_string()] ; "With title")]
#[test_case("multiple_context_vars", "# Documentation", vec!["title:Docs".to_string(), "author:Team".to_string()] ; "Multiple context vars")]
#[test_case("no_context", "# About", vec![] ; "No context")]
// link
#[test_case("multiple_links", "# Links

[Google](https://www.google.com)
[Internal Link](./page.md)
", vec![] ; "Multiple links")]
#[test_case("single_link", "# Home
[Index](./index.md)
", vec![] ; "Single link")]
// image
#[test_case("multiple_images", "# Images

![Alt text](./image.png)
![Remote image](https://example.com/image.jpg)", vec![] ; "Multiple images")]
#[test_case("single_image", "# Single

![Logo](./logo.svg)", vec![] ; "Single image")]
// table
#[test_case("_2x2_table", "# Table

| Column 1 | Column 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |
", vec![] ; "2x2 table")]
// wait https://github.com/johnlepikhin/markdown-ppp/issues/14
#[test_case("toc", "# With Files

 - [readme.txt](/readme.txt)
 - [subdir](/subdir)
     - [nested.md](/subdir/nested.md)
     - [with space in subdir.txt](/subdir/with space in subdir.txt)
 - [test1.md](/test1.md)
 - [test2.html](/test2.html)
 - [with space.txt](/with space.txt)", vec![] ; "TOC")]
#[test]
fn test_md2html(case: &str, md: &str, context_vars: Vec<String>) {
	crate::test::support::init_test_setup();

	let temp_dir = create_test_dir();
	let template_path = temp_dir.path().join("test-template.hbs");

	// Create a minimal template
	fs::write(
		&template_path,
		"<!DOCTYPE html><html>\n<head><title>{{{title}}}</title></head>\n<body>\n{{{body}}}\n</body></html>",
	)
	.expect("Should write template");

	let config = Cofg {
		hbs_path: template_path.to_string_lossy().to_string(),
		templating: crate::cofg::config::CofgTemplating {
			hot_reload: false,
			..Default::default()
		},
		..Cofg::default()
	};

	let html = md2html(md.to_string(), &config, context_vars.clone()).unwrap();
	insta::with_settings!({
		raw_info => &insta::internals::Content::Map(context_vars
			.iter()
			.map(|s| {
				let a: Vec<&str> = s.split(":").collect();
				(
					insta::internals::Content::String(a[0].to_string()),
					insta::internals::Content::String(a[1].to_string()),
				)
			})
			.collect()),
		description => md,
		omit_expression => true,
	}, {
		insta::assert_snapshot!(format!("test_md2html-case-{case}"), html, md);
	});
}

#[test_case(true, "Empty Dir" ; "empty directory")]
#[test_case(false, "With Files" ; "directory with files")]
#[test]
fn test_toc_generation(is_empty: bool, title: &str) {
	crate::test::support::init_test_setup();

	let temp_dir = crate::test::support::PUBLIC_DIR.get().unwrap();

	if !is_empty {
		use build_fs_tree::{dir, file};
		crate::test::support::init_public_dir(dir! {
			"test1.md" => file!("# Test 1"),
			"test2.html" => file!("<h1>Test 2</h1>"),
			"readme.txt" => file!("README"),
			"with space.txt" => file!("Space in filename"),
			"subdir" => dir! {
				"nested.md" => file!("# Nested"),
				"with space in subdir.txt" => file!("Nested space"),
			}
		});
	} else {
		create_dir_all(temp_dir).unwrap();
	}

	let config = Cofg {
		public_path: temp_dir.clone(),
		..Cofg::default()
	};

	let result = markdown::get_toc(
		std::path::Path::new(temp_dir),
		&config,
		Some(title.to_string()),
	);

	if let Err(e) = &result {
		eprintln!("TOC error: {:?}", e);
	}

	assert!(
		result.is_ok(),
		"TOC generation should succeed: {:?}",
		result.err()
	);
	let toc = result.unwrap();
	insta::with_settings!({
		description => title,
		omit_expression => true,
	}, {
		insta::assert_snapshot!(format!("test_toc_generation-{}", if is_empty { "empty" } else { "with_files" }), toc, title);
	});
}

#[test_case("flag:true", "flag", true, false ; "Parse as bool")]
#[test_case("count:123", "count", false, true ; "Parse as number")]
#[test_case("text:hello world", "text", false, false ; "Remain as string")]
#[actix_web::test]
async fn test_context_type_inference_precedence(
	input: &str,
	key: &str,
	expect_bool: bool,
	expect_number: bool,
) {
	let config = Cofg::default();
	let mut context = templating::get_context(&config);

	templating::set_context_value(&mut context, input);
	let data = context.data();

	if expect_bool {
		assert!(
			data.get(key).and_then(|v| v.as_bool()).is_some(),
			"Should parse as bool"
		);
	} else if expect_number {
		assert!(
			data.get(key).and_then(|v| v.as_i64()).is_some(),
			"Should parse as number"
		);
	} else {
		assert!(
			data.get(key).and_then(|v| v.as_str()).is_some(),
			"Should remain as string"
		);
	}
}
