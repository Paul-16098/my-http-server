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

#[test_case("heading_and_text", "# Test\n\nHello world!" ; "Heading and text")]
#[test_case("simple_markdown", "# Welcome\n\nSimple content" ; "Simple markdown")]
#[test_case("h2_heading", "## Section\n\nContent here" ; "H2 heading")]
#[actix_web::test]
async fn test_md2html_basic(case: &str, md: &str) {
	let temp_dir = create_test_dir();
	let template_path = temp_dir.path().join("test-template.hbs");

	// Create a minimal template
	fs::write(
		&template_path,
		"<!DOCTYPE html><html><body>{{{body}}}</body></html>",
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

	let html = md2html(md.to_string(), &config, vec![]).unwrap();
	match case {
		"heading_and_text" => {
			insta::assert_snapshot!(html, @"<!DOCTYPE html><html><body><h1>Test</h1><p>Hello world!</p></body></html>")
		}
		"simple_markdown" => {
			insta::assert_snapshot!(html, @"<!DOCTYPE html><html><body><h1>Welcome</h1><p>Simple content</p></body></html>")
		}
		"h2_heading" => {
			insta::assert_snapshot!(html, @"<!DOCTYPE html><html><body><h2>Section</h2><p>Content here</p></body></html>")
		}
		_ => panic!("Unknown test case: {case}"),
	}
}

#[test_case("with_title", "# Content", vec!["title:Test Page".to_string()] ; "With title")]
#[test_case("multiple_context_vars", "# Documentation", vec!["title:Docs".to_string(), "author:Team".to_string()] ; "Multiple context vars")]
#[test_case("no_context", "# About", vec![] ; "No context")]
#[actix_web::test]
async fn test_md2html_with_context(case: &str, md: &str, context_vars: Vec<String>) {
	let temp_dir = create_test_dir();
	let template_path = temp_dir.path().join("test-template-ctx.hbs");

	// Template that uses context variable
	fs::write(
		&template_path,
		"<!DOCTYPE html><html><head><title>{{title}}</title></head><body>{{{body}}}</body></html>",
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

	let html = md2html(md.to_string(), &config, context_vars).unwrap();
	match case {
		"with_title" => {
			insta::assert_snapshot!(html, @"<!DOCTYPE html><html><head><title>Test Page</title></head><body><h1>Content</h1></body></html>")
		}
		"multiple_context_vars" => {
			insta::assert_snapshot!(html, @"<!DOCTYPE html><html><head><title>Docs</title></head><body><h1>Documentation</h1></body></html>")
		}
		"no_context" => {
			insta::assert_snapshot!(html, @"<!DOCTYPE html><html><head><title></title></head><body><h1>About</h1></body></html>")
		}
		_ => panic!("Unknown test case: {case}"),
	}
}

#[test_case(true, "Empty Dir" ; "empty directory")]
#[test_case(false, "With Files" ; "directory with files")]
#[actix_web::test]
async fn test_toc_generation(is_empty: bool, title: &str) {
	let temp_dir = create_test_dir();

	if !is_empty {
		fs::write(temp_dir.path().join("test1.md"), "# Test 1").expect("Should write test1.md");
		fs::write(temp_dir.path().join("test2.html"), "<h1>Test 2</h1>")
			.expect("Should write test2.html");
		fs::write(temp_dir.path().join("readme.txt"), "README").expect("Should write readme.txt");
	}

	let config = Cofg {
		public_path: temp_dir.path().to_string_lossy().to_string(),
		..Cofg::default()
	};

	let result = markdown::get_toc(temp_dir.path(), &config, Some(title.to_string()));

	if let Err(e) = &result {
		eprintln!("TOC error: {:?}", e);
	}

	assert!(
		result.is_ok(),
		"TOC generation should succeed: {:?}",
		result.err()
	);
	let toc = result.unwrap();
	assert!(toc.contains(title), "TOC should include title");
}

#[test_case(r#"# Links\n\n[Google](https://www.google.com)\n[Internal Link](./page.md)"# ; "Multiple links")]
#[test_case(r#"# Test\n\n[Home](./index.md)"# ; "Single link")]
#[actix_web::test]
async fn test_markdown_with_links(md: &str) {
	let result = markdown::parser_md(md.to_string());
	assert!(
		result.is_ok(),
		"Markdown with links should parse successfully"
	);
}

#[test_case("multiple_links", "# Links\n\n[Google](https://www.google.com)\n[Internal Link](./page.md)\n" ; "Multiple links")]
#[test_case("single_link", "# Home\n\n[Index](./index.md)\n" ; "Single link")]
#[actix_web::test]
async fn test_md2html_with_links_snapshot(case: &str, md: &str) {
	let temp_dir = create_test_dir();
	let template_path = temp_dir.path().join("links-template.hbs");

	fs::write(
		&template_path,
		"<!DOCTYPE html><html><body>{{{body}}}</body></html>",
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

	let html = md2html(md.to_string(), &config, vec![]).unwrap();
	match case {
		"multiple_links" => {
			insta::assert_snapshot!(html, @r###"<!DOCTYPE html><html><body><h1>Links</h1><p><a href="https://www.google.com">Google</a>
<a href="./page.md">Internal Link</a></p></body></html>"###)
		}
		"single_link" => {
			insta::assert_snapshot!(html, @"<!DOCTYPE html><html><body><h1>Home</h1><p><a href=\"./index.md\">Index</a></p></body></html>")
		}
		_ => panic!("Unknown test case: {case}"),
	}
}

#[test_case(r#"# Images\n\n![Alt text](./image.png)\n![Remote image](https://example.com/image.jpg)"# ; "Multiple images")]
#[test_case(r#"# Single\n\n![Logo](./logo.svg)"# ; "Single image")]
#[actix_web::test]
async fn test_markdown_with_images(md: &str) {
	let result = markdown::parser_md(md.to_string());
	assert!(
		result.is_ok(),
		"Markdown with images should parse successfully"
	);
}

#[test_case("multiple_images", "# Images\n\n![Alt text](./image.png)\n![Remote image](https://example.com/image.jpg)\n" ; "Multiple images")]
#[test_case("single_image", "# Logo\n\n![Logo](./logo.svg)\n" ; "Single image")]
#[actix_web::test]
async fn test_md2html_with_images_snapshot(case: &str, md: &str) {
	let temp_dir = create_test_dir();
	let template_path = temp_dir.path().join("images-template.hbs");

	fs::write(
		&template_path,
		"<!DOCTYPE html><html><body>{{{body}}}</body></html>",
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

	let html = md2html(md.to_string(), &config, vec![]).unwrap();
	match case {
		"multiple_images" => {
			insta::assert_snapshot!(html, @r###"<!DOCTYPE html><html><body><h1>Images</h1><p><img src="./image.png" alt="Alt text"></img>
<img src="https://example.com/image.jpg" alt="Remote image"></img></p></body></html>"###)
		}
		"single_image" => {
			insta::assert_snapshot!(html, @"<!DOCTYPE html><html><body><h1>Logo</h1><p><img src=\"./logo.svg\" alt=\"Logo\"></img></p></body></html>")
		}
		_ => panic!("Unknown test case: {case}"),
	}
}

#[test_case("_2x2_table", "# Table\n\n| Column 1 | Column 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |\n| Cell 3   | Cell 4   |\n" ; "2x2 table")]
#[test_case("user_table", "# Users\n\n| Name | Age |\n|------|-----|\n| Alice | 25 |\n| Bob | 30 |\n| Carol | 28 |\n" ; "User table")]
#[actix_web::test]
async fn test_md2html_with_tables_snapshot(case: &str, md: &str) {
	let temp_dir = create_test_dir();
	let template_path = temp_dir.path().join("tables-template.hbs");

	fs::write(
		&template_path,
		"<!DOCTYPE html><html><body>{{{body}}}</body></html>",
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

	let html = md2html(md.to_string(), &config, vec![]).unwrap();
	match case {
		"_2x2_table" => {
			insta::assert_snapshot!(html, @"<!DOCTYPE html><html><body><h1>Table</h1><table><thead><tr><th class=\"markdown-table-align-left\">Column 1</th><th class=\"markdown-table-align-left\">Column 2</th></tr></thead><tbody><tr><td class=\"markdown-table-align-left\">Cell 1</td><td class=\"markdown-table-align-left\">Cell 2</td></tr><tr><td class=\"markdown-table-align-left\">Cell 3</td><td class=\"markdown-table-align-left\">Cell 4</td></tr></tbody></table></body></html>")
		}
		"user_table" => {
			insta::assert_snapshot!(html, @"<!DOCTYPE html><html><body><h1>Users</h1><table><thead><tr><th class=\"markdown-table-align-left\">Name</th><th class=\"markdown-table-align-left\">Age</th></tr></thead><tbody><tr><td class=\"markdown-table-align-left\">Alice</td><td class=\"markdown-table-align-left\">25</td></tr><tr><td class=\"markdown-table-align-left\">Bob</td><td class=\"markdown-table-align-left\">30</td></tr><tr><td class=\"markdown-table-align-left\">Carol</td><td class=\"markdown-table-align-left\">28</td></tr></tbody></table></body></html>")
		}
		_ => panic!("Unknown test case: {case}"),
	}
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

#[test_case("simple_text", "Test" ; "Simple text")]
#[test_case("heading_with_paragraph", "# Hello\n\nWorld" ; "Heading with paragraph")]
#[test_case("horizontal_rule", "---\n\nHorizontal rule" ; "Horizontal rule")]
#[actix_web::test]
async fn test_empty_template_data(case: &str, md: &str) {
	let temp_dir = create_test_dir();
	let template_path = temp_dir.path().join("empty-ctx.hbs");

	fs::write(&template_path, "<html>{{{body}}}</html>").expect("Should write template");

	let config = Cofg {
		hbs_path: template_path.to_string_lossy().to_string(),
		..Cofg::default()
	};

	let html = md2html(md.to_string(), &config, vec![]).unwrap();
	match case {
		"simple_text" => insta::assert_snapshot!(html, @"<html><p>Test</p></html>"),
		"heading_with_paragraph" => {
			insta::assert_snapshot!(html, @"<html><h1>Hello</h1><p>World</p></html>")
		}
		"horizontal_rule" => {
			insta::assert_snapshot!(html, @"<html><hr></hr><p>Horizontal rule</p></html>")
		}
		_ => panic!("Unknown test case: {case}"),
	}
}
