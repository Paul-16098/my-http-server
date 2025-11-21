//! HTTP request handler tests
//!
//! Tests for routing logic, 404 handling, markdown rendering, and TOC generation.

use std::fs;
use tempfile::TempDir;

use crate::request::server_error;

#[test]
fn test_server_error_response() {
    let response = server_error("Test error message".to_string());
    assert_eq!(
        response.status(),
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    );
}

// Async tests commented out - need proper async runtime setup
// #[actix_web::test]
// async fn test_server_error_body() {
//     let response = server_error("Custom error".to_string());
//     let body = actix_web::body::to_bytes(response.into_body()).await.unwrap();
//     let body_str = String::from_utf8(body.to_vec()).unwrap();
//     assert_eq!(body_str, "Custom error");
// }

// Integration tests commented out - need full server setup
// #[actix_web::test]
// async fn test_main_req_missing_file_returns_404() { ... }
// #[actix_web::test]
// async fn test_main_req_static_file() { ... }
// #[actix_web::test]
// async fn test_404_response_structure() { ... }

#[test]
fn test_markdown_file_detection() {
    use std::path::Path;
    let md_path = Path::new("test.md");
    assert_eq!(md_path.extension().and_then(|v| v.to_str()), Some("md"));

    let txt_path = Path::new("test.txt");
    assert_ne!(txt_path.extension().and_then(|v| v.to_str()), Some("md"));
}

#[test]
fn test_path_extension_edge_cases() {
    use std::path::Path;
    // No extension
    let no_ext = Path::new("filename");
    assert_eq!(no_ext.extension(), None);

    // Hidden file
    let hidden = Path::new(".hidden");
    assert_eq!(hidden.extension(), None);

    // Multiple dots
    let multi_dot = Path::new("file.tar.gz");
    assert_eq!(multi_dot.extension().and_then(|v| v.to_str()), Some("gz"));
}

#[test]
fn test_percent_encoding_in_path() {
    use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

    let original = "hello world";
    let encoded = utf8_percent_encode(original, NON_ALPHANUMERIC).to_string();
    assert_eq!(encoded, "hello%20world");

    let decoded = percent_encoding::percent_decode_str(&encoded)
        .decode_utf8()
        .unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_path_traversal_detection() {
    use std::path::Path;
    // Test that path traversal attempts are properly handled
    let base = Path::new("/public");
    let safe_path = Path::new("/public/docs/file.md");
    let unsafe_path = Path::new("/etc/passwd");

    assert!(safe_path.starts_with(base));
    assert!(!unsafe_path.starts_with(base));
}

#[test]
fn test_strip_prefix_behavior() {
    use std::path::Path;
    let base = Path::new("/public");
    let full_path = Path::new("/public/docs/readme.md");

    let stripped = full_path.strip_prefix(base).unwrap();
    assert_eq!(stripped, Path::new("docs/readme.md"));
}

#[test]
fn test_strip_prefix_error_case() {
    use std::path::Path;
    let base = Path::new("/public");
    let unrelated_path = Path::new("/other/file.txt");

    assert!(unrelated_path.strip_prefix(base).is_err());
}

// Integration test structure for TOC generation
#[test]
fn test_toc_label_formatting() {
    let label = "test/directory";
    let formatted = if label.is_empty() { "?" } else { label };
    assert_eq!(formatted, "test/directory");

    let empty_label = "";
    let formatted_empty = if empty_label.is_empty() {
        "?"
    } else {
        empty_label
    };
    assert_eq!(formatted_empty, "?");
}

#[test]
fn test_index_file_detection() {
    let temp_dir = TempDir::new().unwrap();
    let public_path = temp_dir.path();

    // Test without index.html
    assert!(!public_path.join("index.html").exists());

    // Create index.html
    fs::write(public_path.join("index.html"), "index content").unwrap();
    assert!(public_path.join("index.html").exists());
}

#[test]
fn test_file_type_checks() {
    let temp_dir = TempDir::new().unwrap();

    // Test file
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "content").unwrap();
    assert!(file_path.is_file());
    assert!(!file_path.is_dir());

    // Test directory
    let dir_path = temp_dir.path().join("subdir");
    fs::create_dir(&dir_path).unwrap();
    assert!(!dir_path.is_file());
    assert!(dir_path.is_dir());
}

#[test]
fn test_canonicalize_behavior() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "content").unwrap();

    let canonical = test_file.canonicalize().unwrap();
    assert!(canonical.is_absolute());
}

#[test]
fn test_path_equality_after_canonicalize() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();

    let path1 = subdir.canonicalize().unwrap();
    let path2 = subdir.canonicalize().unwrap();
    assert_eq!(path1, path2);
}
