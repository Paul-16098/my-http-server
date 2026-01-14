//! Configuration tests - fixtures and config loading/precedence validation
//!
//! WHY: Ensure configuration system works correctly:
//! - Default config loads properly
//! - Config caching/reloading respects hot_reload flag
//! - XDG path resolution works correctly
//! - Config precedence (built-in → file → env → CLI) works as expected

use crate::cofg::{
    cli,
    config::{Cofg, CofgAddrs},
};

#[actix_web::test]
async fn test_default_config_loads() {
    let config = Cofg::default();

    // Verify basic structure is present
    assert!(
        !config.public_path.is_empty(),
        "public_path should not be empty"
    );
    assert!(
        !config.page_404_path.is_empty(),
        "page_404_path should not be empty"
    );
    assert!(!config.hbs_path.is_empty(), "hbs_path should not be empty");
}

#[actix_web::test]
async fn test_config_parsing_from_yaml() {
    let yaml_content = r#"
addrs:
  ip: "0.0.0.0"
  port: 3000

tls:
  enable: false
  cert: "./cert.pem"
  key: "./key.pem"

middleware:
  normalize_path: true
  compress: true
  logger:
    enabling: true
    format: "%a %t %r %s %b"
  http_base_authentication:
    enable: false
    users: null
  ip_filter:
    enable: false
    rules: []
  rate_limiting:
    enable: false
    seconds_per_request: 1
    burst_size: 10

api:
  enable: false
  allow_edit: false

templating:
  value: null
  hot_reload: false

toc:
  ext:
    - md
    - html
  ig:
    - node_modules
    - .git

public_path: "./public"
page_404_path: "./meta/404.html"
hbs_path: "./meta/html-t.hbs"
"#;

    let config = Cofg::new_from_str(yaml_content).expect("Should parse test YAML");

    assert_eq!(config.addrs.ip, "0.0.0.0");
    assert_eq!(config.addrs.port, 3000);
    assert_eq!(config.tls.cert, "./cert.pem");
    assert_eq!(config.tls.key, "./key.pem");
    assert_eq!(config.public_path, "./public");
}

#[actix_web::test]
async fn test_xdg_paths_available() {
    // This test checks that XDG path resolution doesn't panic
    let xdg_paths = Cofg::get_xdg_paths();

    // On most systems, XDG paths should be available
    // But we don't assert it exists since it might not be available in all environments
    if let Some(paths) = xdg_paths {
        assert!(
            !paths.cofg.as_os_str().is_empty(),
            "Config path should not be empty"
        );
        assert!(
            !paths.page_404.as_os_str().is_empty(),
            "404 page path should not be empty"
        );
        assert!(
            !paths.template_hbs.as_os_str().is_empty(),
            "Template path should not be empty"
        );
    }
}

#[actix_web::test]
async fn test_config_clone() {
    let config1 = Cofg::default();
    let config2 = config1.clone();

    // Verify cloning works and creates equal configs
    assert_eq!(config1, config2, "Cloned config should equal original");
    assert_eq!(config1.addrs.ip, config2.addrs.ip);
    assert_eq!(config1.addrs.port, config2.addrs.port);
}

#[actix_web::test]
async fn test_empty_toc_extensions() {
    let yaml_content = r#"
addrs:
  ip: "127.0.0.1"
  port: 8080

tls:
  enable: false
  cert: ""
  key: ""

middleware:
  normalize_path: true
  compress: true
  logger:
    enabling: true
    format: ""
  http_base_authentication:
    enable: false
    users: null
  ip_filter:
    enable: false
    rules: []
  rate_limiting:
    enable: false
    seconds_per_request: 1
    burst_size: 10

api:
  enable: false
  allow_edit: false

templating:
  value: null
  hot_reload: false

toc:
  ext: []
  ig: []

public_path: "./public"
page_404_path: "./meta/404.html"
hbs_path: "./meta/html-t.hbs"
"#;

    let config = Cofg::new_from_str(yaml_content).expect("Should parse YAML with empty extensions");

    assert!(config.toc.ext.is_empty(), "TOC extensions should be empty");
    assert!(config.toc.ig.is_empty(), "TOC ignore list should be empty");
}

#[actix_web::test]
async fn test_middleware_flags() {
    let yaml_content = r#"
addrs:
  ip: "127.0.0.1"
  port: 8080

tls:
  enable: false
  cert: ""
  key: ""

middleware:
  normalize_path: false
  compress: false
  logger:
    enabling: false
    format: ""
  http_base_authentication:
    enable: true
    users: null
  ip_filter:
    enable: true
    rules: []
  rate_limiting:
    enable: true
    seconds_per_request: 2
    burst_size: 5

api:
  enable: false
  allow_edit: false

templating:
  value: null
  hot_reload: true

toc:
  ext:
    - md
  ig: []

public_path: "./public"
page_404_path: "./meta/404.html"
hbs_path: "./meta/html-t.hbs"
"#;

    let config = Cofg::new_from_str(yaml_content).expect("Should parse YAML");

    assert!(
        !config.middleware.normalize_path,
        "normalize_path should be false"
    );
    assert!(!config.middleware.compress, "compress should be false");
    assert!(
        !config.middleware.logger.enabling,
        "logger should be disabled"
    );
    assert!(
        config.middleware.http_base_authentication.enable,
        "auth should be enabled"
    );
    assert!(
        config.middleware.ip_filter.enable,
        "ip_filter should be enabled"
    );
    assert!(
        config.middleware.rate_limiting.enable,
        "rate_limiting should be enabled"
    );
    assert_eq!(config.middleware.rate_limiting.seconds_per_request, 2);
    assert_eq!(config.middleware.rate_limiting.burst_size, 5);
    assert!(config.templating.hot_reload, "hot_reload should be true");
}

/// Helper function to create a temporary directory for test fixtures
pub(crate) fn create_test_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

/// Initialize global config for all test suites.
///
/// Uses `std::sync::Once` to ensure thread-safe initialization that runs exactly once per process.
/// This prevents race conditions when tests run in parallel and avoids redundant initialization.
///
/// WHY: Tests trigger global config initialization which can cause:
/// - Network calls to GitHub API (with github_emojis feature)
/// - File I/O for XDG config directories
/// - Race conditions if multiple tests initialize simultaneously
///
/// Emoji stub file location:
/// - Stored in OS temp directory (not project root) to avoid CI/CD pollution
/// - Path: std::env::temp_dir()/my-http-server-test-emojis.json
/// - Auto-managed by OS (cleaned up according to OS temp file policies)
///
/// NOTE: `Once::call_once` guarantees the closure runs only once even across multiple test runs
/// in the same process. This is intentional - tests share this global state for efficiency.
/// For test isolation, run tests in separate processes or use `--test-threads=1`.
pub(crate) fn init_test_config() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        use clap::Parser;

        let args = cli::Args::try_parse_from(["test"].as_ref()).unwrap_or_else(|_| cli::Args::parse());
        let _ = Cofg::init_global(&args, true); // true = skip XDG to avoid file I/O

        // Create minimal emojis.json stub in temp directory to prevent GitHub API calls
        // WHY: The github_emojis feature would otherwise fetch emoji data from GitHub API,
        // causing tests to hang or fail in CI environments without network access.
        // Stored in temp directory (not project root) to avoid polluting repository.
        #[cfg(feature = "github_emojis")]
        {
            let temp_dir = std::env::temp_dir();
            let emoji_path = temp_dir.join("my-http-server-test-emojis.json");
            if !emoji_path.exists() {
                let _ = std::fs::write(emoji_path, r#"{"unicode":{},"else":{}}"#);
            }
        }
    });
}

/// Test conversion from CLI args to Config Addrs
#[test]
fn test_cli_args_to_config_addrs() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        CofgAddrs::try_from(cli::Args {
            ip: Some("127.0.0.1".to_string()),
            port: Some(1634),
            ..Default::default()
        })?,
        CofgAddrs {
            ip: "127.0.0.1".to_string(),
            port: 1634
        }
    );
    Ok(())
}

/// Test conversion from CLI args reference to Config Addrs
#[test]
fn test_cli_args_ref_to_config_addrs() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        CofgAddrs::try_from(&cli::Args {
            ip: Some("127.0.0.1".to_string()),
            port: Some(1634),
            ..Default::default()
        })?,
        CofgAddrs {
            ip: "127.0.0.1".to_string(),
            port: 1634
        }
    );
    Ok(())
}

#[test]
fn test_cli_args_ref_to_config_addrs_error() {
    assert_eq!(
        CofgAddrs::try_from(&cli::Args {
            ip: Some("127.0.0.1".to_string()),
            port: None,
            ..Default::default()
        })
        .err()
        .unwrap()
        .to_string(),
        crate::error::AppError::OtherError("ip or port is none".to_string()).to_string()
    );
}

/// Test conversion from CLI args to Config Addrs error
#[test]
fn test_cli_args_to_config_addrs_error() {
    assert_eq!(
        CofgAddrs::try_from(cli::Args {
            ip: Some("127.0.0.1".to_string()),
            port: None,
            ..Default::default()
        })
        .err()
        .unwrap()
        .to_string(),
        crate::error::AppError::OtherError("ip or port is none".to_string()).to_string()
    );
}
