use crate::cofg::config::Cofg;
use crate::parser::markdown::{ get_toc, parser_md, TocCacheKey };

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

#[test]
fn test_toc_cache_key_from_dir() {
  let temp_dir = tempfile::tempdir().unwrap();
  let dir = temp_dir.path();

  let key = TocCacheKey::from_dir(dir, Some("Test Title".to_string())).unwrap();
  assert_eq!(key.title, Some("Test Title".to_string()));
  assert_eq!(key.dir, dir.to_path_buf());
}
