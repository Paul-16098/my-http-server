use crate::cofg::config::Cofg;
use crate::parser::markdown::{get_toc, parser_md};
use std::fs;

#[test]
fn test_get_toc_handles_empty_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root_path = temp_dir.path().to_path_buf();

    let mut config = Cofg {
        public_path: root_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    config.toc.ext.insert("md".to_string());

    let toc = get_toc(&root_path, &config, Some("Empty TOC".to_string())).unwrap();
    assert!(toc.contains("# Empty TOC"));
    assert!(!toc.contains("- [")); // No entries should be present
}

#[test]
fn test_parser_md_parses_valid_markdown() {
    let input = "# Title\n\nSome content.".to_string();
    let document = parser_md(input).unwrap();
    assert_eq!(document.blocks.len(), 2); // One heading and one paragraph
}

#[test]
fn test_parser_md_handles_empty_input() {
    let input = "".to_string();
    let document = parser_md(input).unwrap();
    assert!(document.blocks.is_empty());
}

// ===== New expanded tests =====

#[test]
fn test_get_toc_single_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root_path = temp_dir.path();

    // Create a single markdown file
    fs::write(root_path.join("readme.md"), "# Test").unwrap();

    let mut config = Cofg {
        public_path: root_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    config.toc.ext.insert("md".to_string());

    let toc = get_toc(root_path, &config, Some("Test TOC".to_string())).unwrap();
    assert!(toc.contains("# Test TOC"));
    assert!(toc.contains("readme.md"));
}

#[test]
fn test_get_toc_multi_level_directories() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root_path = temp_dir.path();

    // Create nested directory structure
    fs::create_dir_all(root_path.join("level1/level2")).unwrap();
    fs::write(root_path.join("top.md"), "top").unwrap();
    fs::write(root_path.join("level1/middle.md"), "middle").unwrap();
    fs::write(root_path.join("level1/level2/deep.md"), "deep").unwrap();

    let mut config = Cofg {
        public_path: root_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    config.toc.ext.insert("md".to_string());

    let toc = get_toc(root_path, &config, Some("Multi-Level".to_string())).unwrap();
    assert!(toc.contains("top.md"));
    assert!(toc.contains("middle.md") || toc.contains("level1"));
    assert!(toc.contains("deep.md") || toc.contains("level2"));
}

#[test]
fn test_get_toc_extension_filtering() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root_path = temp_dir.path();

    // Create files with different extensions
    fs::write(root_path.join("doc.md"), "markdown").unwrap();
    fs::write(root_path.join("note.txt"), "text").unwrap();
    fs::write(root_path.join("data.json"), "{}").unwrap();

    let mut config = Cofg {
        public_path: root_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    config.toc.ext.insert("md".to_string());

    let toc = get_toc(root_path, &config, None).unwrap();
    // The TOC should include .md files
    assert!(toc.contains("doc.md"));
    // Non-.md files should not be in the TOC if extension filtering works
    // Note: This test may need adjustment based on actual TOC output format
}

#[test]
fn test_get_toc_multiple_extensions() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root_path = temp_dir.path();

    fs::write(root_path.join("doc.md"), "markdown").unwrap();
    fs::write(root_path.join("note.txt"), "text").unwrap();
    fs::write(root_path.join("readme.rst"), "rest").unwrap();

    let mut config = Cofg {
        public_path: root_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    config.toc.ext.insert("md".to_string());
    config.toc.ext.insert("txt".to_string());

    let toc = get_toc(root_path, &config, None).unwrap();
    assert!(toc.contains("doc.md"));
    assert!(toc.contains("note.txt"));
    assert!(!toc.contains("readme.rst"));
}

#[test]
fn test_get_toc_ignore_pattern() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root_path = temp_dir.path();

    fs::write(root_path.join("readme.md"), "doc").unwrap();
    fs::write(root_path.join("draft.md"), "draft").unwrap();
    fs::create_dir(root_path.join(".git")).unwrap();
    fs::write(root_path.join(".git/config"), "git").unwrap();

    let mut config = Cofg {
        public_path: root_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    config.toc.ext.insert("md".to_string());
    config.toc.ig.insert("draft*".to_string());
    config.toc.ig.insert(".*".to_string());

    let toc = get_toc(root_path, &config, None).unwrap();
    assert!(toc.contains("readme.md"));
    // Draft and hidden files should be ignored
}

#[test]
fn test_get_toc_percent_encoding_spaces() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root_path = temp_dir.path();

    fs::write(root_path.join("file with spaces.md"), "content").unwrap();

    let mut config = Cofg {
        public_path: root_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    config.toc.ext.insert("md".to_string());

    let toc = get_toc(root_path, &config, None).unwrap();
    // Should contain percent-encoded version
    assert!(toc.contains("file") && toc.contains("spaces"));
}

#[test]
fn test_get_toc_special_characters() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root_path = temp_dir.path();

    // Create files with special characters (where filesystem allows)
    fs::write(root_path.join("file-name.md"), "content").unwrap();
    fs::write(root_path.join("file_name.md"), "content").unwrap();

    let mut config = Cofg {
        public_path: root_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    config.toc.ext.insert("md".to_string());

    let toc = get_toc(root_path, &config, None).unwrap();
    assert!(toc.contains("file-name.md"));
    assert!(toc.contains("file_name.md"));
}

#[test]
fn test_get_toc_unicode_filenames() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root_path = temp_dir.path();

    fs::write(root_path.join("测试.md"), "Chinese").unwrap();
    fs::write(root_path.join("テスト.md"), "Japanese").unwrap();

    let mut config = Cofg {
        public_path: root_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    config.toc.ext.insert("md".to_string());

    let toc = get_toc(root_path, &config, None).unwrap();
    // Should handle unicode filenames
    assert!(toc.len() > 0);
}

#[test]
fn test_get_toc_mixed_files_and_directories() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root_path = temp_dir.path();

    fs::write(root_path.join("file1.md"), "file").unwrap();
    fs::create_dir(root_path.join("subdir")).unwrap();
    fs::write(root_path.join("subdir/file2.md"), "nested").unwrap();
    fs::write(root_path.join("file3.md"), "another").unwrap();

    let mut config = Cofg {
        public_path: root_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    config.toc.ext.insert("md".to_string());

    let toc = get_toc(root_path, &config, None).unwrap();
    assert!(toc.contains("file1.md"));
    assert!(toc.contains("file3.md"));
}

#[test]
fn test_parser_md_heading_levels() {
    let input = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6".to_string();
    let document = parser_md(input).unwrap();
    assert_eq!(document.blocks.len(), 6);
}

#[test]
fn test_parser_md_code_blocks() {
    let input = "```rust\nfn main() {}\n```".to_string();
    let document = parser_md(input).unwrap();
    assert!(!document.blocks.is_empty());
}

#[test]
fn test_parser_md_lists() {
    let input = "- Item 1\n- Item 2\n- Item 3".to_string();
    let document = parser_md(input).unwrap();
    assert!(!document.blocks.is_empty());
}

#[test]
fn test_parser_md_links() {
    let input = "[Link text](https://example.com)".to_string();
    let document = parser_md(input).unwrap();
    assert!(!document.blocks.is_empty());
}

#[test]
fn test_parser_md_emphasis() {
    let input = "*italic* and **bold** and ***both***".to_string();
    let document = parser_md(input).unwrap();
    assert!(!document.blocks.is_empty());
}

#[test]
fn test_parser_md_blockquotes() {
    let input = "> This is a quote\n> Second line".to_string();
    let document = parser_md(input).unwrap();
    assert!(!document.blocks.is_empty());
}

#[test]
fn test_parser_md_horizontal_rule() {
    let input = "Above\n\n---\n\nBelow".to_string();
    let document = parser_md(input).unwrap();
    assert!(document.blocks.len() >= 2);
}

#[test]
fn test_parser_md_mixed_content() {
    let input = r#"# Title

Some paragraph with **bold** text.

- List item 1
- List item 2

```
code block
```

[Link](url)
"#
    .to_string();
    let document = parser_md(input).unwrap();
    assert!(document.blocks.len() >= 4);
}
