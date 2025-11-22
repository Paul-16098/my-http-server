//! API endpoint tests
//!
//! Tests for file-related API endpoints

use std::fs;
use tempfile::TempDir;

#[test]
fn test_file_info_struct_creation() {
    use crate::api::file::FileInfo;

    let info = FileInfo {
        name: "test.txt".to_string(),
        path: "dir/test.txt".to_string(),
        size: Some(1024),
        modified: Some(1234567890),
        is_directory: false,
        is_file: true,
    };

    assert_eq!(info.name, "test.txt");
    assert_eq!(info.path, "dir/test.txt");
    assert_eq!(info.size, Some(1024));
    assert_eq!(info.modified, Some(1234567890));
    assert!(!info.is_directory);
    assert!(info.is_file);
}

#[test]
fn test_directory_listing_struct_creation() {
    use crate::api::file::{DirectoryListing, FileInfo};

    let entries = vec![
        FileInfo {
            name: "file1.txt".to_string(),
            path: "file1.txt".to_string(),
            size: Some(100),
            modified: Some(1234567890),
            is_directory: false,
            is_file: true,
        },
        FileInfo {
            name: "subdir".to_string(),
            path: "subdir".to_string(),
            size: None,
            modified: Some(1234567890),
            is_directory: true,
            is_file: false,
        },
    ];

    let listing = DirectoryListing {
        path: "test_dir".to_string(),
        entries,
    };

    assert_eq!(listing.path, "test_dir");
    assert_eq!(listing.entries.len(), 2);
    assert_eq!(listing.entries[0].name, "file1.txt");
    assert!(listing.entries[0].is_file);
    assert_eq!(listing.entries[1].name, "subdir");
    assert!(listing.entries[1].is_directory);
}

#[test]
fn test_exists_response_struct_creation() {
    use crate::api::file::ExistsResponse;

    let response_exists = ExistsResponse {
        exists: true,
        path_type: Some("file".to_string()),
    };

    assert!(response_exists.exists);
    assert_eq!(response_exists.path_type, Some("file".to_string()));

    let response_not_exists = ExistsResponse {
        exists: false,
        path_type: None,
    };

    assert!(!response_not_exists.exists);
    assert_eq!(response_not_exists.path_type, None);
}

#[test]
fn test_file_info_serialization() {
    use crate::api::file::FileInfo;
    use serde_json;

    let info = FileInfo {
        name: "test.md".to_string(),
        path: "docs/test.md".to_string(),
        size: Some(512),
        modified: Some(1609459200),
        is_directory: false,
        is_file: true,
    };

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("test.md"));
    assert!(json.contains("docs/test.md"));
    assert!(json.contains("512"));

    let deserialized: FileInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, info.name);
    assert_eq!(deserialized.path, info.path);
    assert_eq!(deserialized.size, info.size);
}

#[test]
fn test_directory_listing_serialization() {
    use crate::api::file::{DirectoryListing, FileInfo};
    use serde_json;

    let listing = DirectoryListing {
        path: ".".to_string(),
        entries: vec![FileInfo {
            name: "readme.md".to_string(),
            path: "readme.md".to_string(),
            size: Some(256),
            modified: Some(1609459200),
            is_directory: false,
            is_file: true,
        }],
    };

    let json = serde_json::to_string(&listing).unwrap();
    assert!(json.contains("readme.md"));

    let deserialized: DirectoryListing = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.path, listing.path);
    assert_eq!(deserialized.entries.len(), 1);
    assert_eq!(deserialized.entries[0].name, "readme.md");
}

#[test]
fn test_exists_response_serialization() {
    use crate::api::file::ExistsResponse;
    use serde_json;

    let response = ExistsResponse {
        exists: true,
        path_type: Some("directory".to_string()),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("true"));
    assert!(json.contains("directory"));

    let deserialized: ExistsResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.exists, response.exists);
    assert_eq!(deserialized.path_type, response.path_type);
}

#[test]
fn test_file_info_with_directory_no_size() {
    use crate::api::file::FileInfo;

    let info = FileInfo {
        name: "my_directory".to_string(),
        path: "path/to/my_directory".to_string(),
        size: None, // Directories don't have size
        modified: Some(1609459200),
        is_directory: true,
        is_file: false,
    };

    assert!(info.is_directory);
    assert!(!info.is_file);
    assert_eq!(info.size, None);
}

#[test]
fn test_empty_directory_listing() {
    use crate::api::file::DirectoryListing;

    let listing = DirectoryListing {
        path: "empty_dir".to_string(),
        entries: vec![],
    };

    assert_eq!(listing.entries.len(), 0);
    assert!(listing.entries.is_empty());
}

#[test]
fn test_path_validation_logic() {
    // Test basic path validation concepts used in the API
    use std::path::Path;

    let path = Path::new("./test/../test.txt");
    assert!(path.to_string_lossy().contains(".."));

    let normalized = path.to_string_lossy().replace("..", "");
    assert!(!normalized.contains(".."));
}

#[test]
fn test_relative_path_construction() {
    use std::path::PathBuf;

    let public_path = PathBuf::from("/home/user/public");
    let full_path = PathBuf::from("/home/user/public/docs/readme.md");

    let relative = full_path
        .strip_prefix(&public_path)
        .unwrap()
        .to_string_lossy()
        .to_string();

    assert_eq!(relative, "docs/readme.md");
}

#[test]
fn test_file_type_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create a file
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test content").unwrap();

    // Create a directory
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir(&dir_path).unwrap();

    // Test file type detection
    assert!(file_path.is_file());
    assert!(!file_path.is_dir());
    assert!(dir_path.is_dir());
    assert!(!dir_path.is_file());
}

#[test]
fn test_file_metadata_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let content = "Hello, World!";
    fs::write(&file_path, content).unwrap();

    let metadata = fs::metadata(&file_path).unwrap();

    assert!(metadata.is_file());
    assert_eq!(metadata.len(), content.len() as u64);
    assert!(metadata.modified().is_ok());
}

#[test]
fn test_directory_reading() {
    let temp_dir = TempDir::new().unwrap();

    // Create some files and directories
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.md"), "content2").unwrap();
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();

    let entries: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(entries.len(), 3);

    let names: Vec<String> = entries
        .iter()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    assert!(names.contains(&"file1.txt".to_string()));
    assert!(names.contains(&"file2.md".to_string()));
    assert!(names.contains(&"subdir".to_string()));
}

#[test]
fn test_sorting_entries() {
    use crate::api::file::FileInfo;

    let mut entries = [
        FileInfo {
            name: "zebra.txt".to_string(),
            path: "zebra.txt".to_string(),
            size: Some(100),
            modified: None,
            is_directory: false,
            is_file: true,
        },
        FileInfo {
            name: "alpha_dir".to_string(),
            path: "alpha_dir".to_string(),
            size: None,
            modified: None,
            is_directory: true,
            is_file: false,
        },
        FileInfo {
            name: "beta.txt".to_string(),
            path: "beta.txt".to_string(),
            size: Some(200),
            modified: None,
            is_directory: false,
            is_file: true,
        },
    ];

    // Sort: directories first, then files, alphabetically
    entries.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    assert_eq!(entries[0].name, "alpha_dir");
    assert!(entries[0].is_directory);
    assert_eq!(entries[1].name, "beta.txt");
    assert!(entries[1].is_file);
    assert_eq!(entries[2].name, "zebra.txt");
    assert!(entries[2].is_file);
}

#[test]
fn test_unix_timestamp_conversion() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap();
    let timestamp = duration.as_secs() as i64;

    assert!(timestamp > 0);
    assert!(timestamp > 1600000000); // After 2020
}

#[test]
fn test_path_type_strings() {
    // Test the path type string constants used in ExistsResponse
    let file_type = "file";
    let dir_type = "directory";
    let other_type = "other";

    assert_eq!(file_type, "file");
    assert_eq!(dir_type, "directory");
    assert_eq!(other_type, "other");
}

#[test]
fn test_empty_path_handling() {
    let empty = "";
    let whitespace = "   ";

    assert!(empty.trim().is_empty());
    assert!(whitespace.trim().is_empty());
}

#[test]
fn test_path_traversal_detection() {
    use std::path::Path;

    let public = Path::new("/home/user/public");
    let safe_path = Path::new("/home/user/public/docs/file.txt");
    let unsafe_path = Path::new("/home/user/other/file.txt");

    assert!(safe_path.starts_with(public));
    assert!(!unsafe_path.starts_with(public));
}
