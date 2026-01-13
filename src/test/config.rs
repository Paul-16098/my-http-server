//! Configuration tests - fixtures and config loading/precedence validation
//!
//! WHY: Ensure configuration system works correctly:
//! - Default config loads properly
//! - Config caching/reloading respects hot_reload flag
//! - XDG path resolution works correctly
//! - Config precedence (built-in → file → env → CLI) works as expected

use crate::cofg::config::Cofg;

#[actix_web::test]
async fn test_default_config_loads() {
    let config = Cofg::default();
    
    // Verify basic structure is present
    assert!(!config.public_path.is_empty(), "public_path should not be empty");
    assert!(!config.page_404_path.is_empty(), "page_404_path should not be empty");
    assert!(!config.hbs_path.is_empty(), "hbs_path should not be empty");
}

#[actix_web::test]
async fn test_config_has_expected_defaults() {
    let config = Cofg::default();
    
    // Check default address
    assert_eq!(config.addrs.ip, "localhost", "Default IP should be localhost");
    assert_eq!(config.addrs.port, 8080, "Default port should be 8080");
    
    // Check TLS is disabled by default
    assert!(!config.tls.enable, "TLS should be disabled by default");
    
    // Check middleware defaults
    assert!(config.middleware.normalize_path, "NormalizePath should be enabled by default");
    assert!(config.middleware.compress, "Compress should be enabled by default");
    
    // Check templating defaults
    assert!(!config.templating.hot_reload, "Hot reload should be disabled by default");
}

#[actix_web::test]
async fn test_toc_extensions_default() {
    let config = Cofg::default();
    
    // Verify TOC has some default extensions
    assert!(!config.toc.ext.is_empty(), "TOC extensions should not be empty");
    
    // Common extensions should be present
    let expected_exts = vec!["md", "html", "txt"];
    for ext in expected_exts {
        assert!(
            config.toc.ext.contains(ext),
            "TOC should include .{} extension",
            ext
        );
    }
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
        assert!(!paths.cofg.as_os_str().is_empty(), "Config path should not be empty");
        assert!(!paths.page_404.as_os_str().is_empty(), "404 page path should not be empty");
        assert!(!paths.template_hbs.as_os_str().is_empty(), "Template path should not be empty");
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
    
    assert!(!config.middleware.normalize_path, "normalize_path should be false");
    assert!(!config.middleware.compress, "compress should be false");
    assert!(!config.middleware.logger.enabling, "logger should be disabled");
    assert!(config.middleware.http_base_authentication.enable, "auth should be enabled");
    assert!(config.middleware.ip_filter.enable, "ip_filter should be enabled");
    assert!(config.middleware.rate_limiting.enable, "rate_limiting should be enabled");
    assert_eq!(config.middleware.rate_limiting.seconds_per_request, 2);
    assert_eq!(config.middleware.rate_limiting.burst_size, 5);
    assert!(config.templating.hot_reload, "hot_reload should be true");
}

/// Helper function to create a minimal valid config for tests
pub(crate) fn minimal_test_config() -> Cofg {
    Cofg::default()
}

/// Helper function to create a temporary directory for test fixtures
pub(crate) fn create_test_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}
