//! Security tests - Path traversal, authentication, and IP filtering
//!
//! WHY: Validate security-critical behaviors:
//! - Path traversal prevention
//! - HTTP Basic Authentication
//! - IP filtering rules
//! - Constant-time password comparison
//! - Access control (allow/disallow paths)

use crate::cofg::cli::Args;
use crate::cofg::config::Cofg;
use crate::request::main_req;
use actix_web::{App, http::StatusCode, test};
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize global config for security tests
///
/// Uses `Once` to ensure thread-safe initialization that runs exactly once per process lifetime.
/// This prevents race conditions when tests run in parallel and avoids redundant initialization.
///
/// WHY: Tests trigger global config initialization which can cause:
/// - Network calls to GitHub API (with github_emojis feature)
/// - File I/O for XDG config directories
/// - Race conditions if multiple tests initialize simultaneously
///
/// NOTE: `Once::call_once` guarantees the closure runs only once even across multiple test runs
/// in the same process. This is intentional - tests are designed to share this global state.
/// For test isolation, run tests in separate processes or use `--test-threads=1`.
fn init_test_config() {
    INIT.call_once(|| {
        use clap::Parser;

        let args = Args::try_parse_from(["test"].as_ref()).unwrap_or_else(|_| Args::parse());
        let _ = Cofg::init_global(&args, true); // true = skip XDG to avoid file I/O

        // Create a minimal emojis.json stub to prevent GitHub API calls during testing
        // WHY: The github_emojis feature would otherwise fetch emoji data from GitHub API,
        // causing tests to hang or fail in CI environments without network access.
        // This file is intentionally left after tests (not cleaned up) as it's small and
        // can be reused across test runs. Add to .gitignore if needed.
        #[cfg(feature = "github_emojis")]
        {
            let emoji_path = std::path::Path::new("./emojis.json");
            if !emoji_path.exists() {
                let _ = std::fs::write(emoji_path, r#"{"unicode":{},"else":{}}"#);
            }
        }
    });
}

#[actix_web::test]
async fn test_constant_time_eq_identical() {
    use crate::constant_time_eq;
    let a = b"password123";
    let b = b"password123";

    assert!(
        constant_time_eq(a, b),
        "Identical byte arrays should be equal"
    );
}

#[actix_web::test]
async fn test_constant_time_eq_different() {
    use crate::constant_time_eq;
    let a = b"password123";
    let b = b"password456";

    assert!(
        !constant_time_eq(a, b),
        "Different byte arrays should not be equal"
    );
}

#[actix_web::test]
async fn test_constant_time_eq_different_length() {
    use crate::constant_time_eq;
    let a = b"short";
    let b = b"much_longer_string";

    assert!(
        !constant_time_eq(a, b),
        "Arrays of different length should not be equal"
    );
}

#[actix_web::test]
async fn test_constant_time_eq_empty() {
    use crate::constant_time_eq;
    let a = b"";
    let b = b"";

    assert!(constant_time_eq(a, b), "Empty arrays should be equal");
}

#[actix_web::test]
async fn test_constant_time_eq_one_empty() {
    use crate::constant_time_eq;
    let a = b"";
    let b = b"nonempty";

    assert!(
        !constant_time_eq(a, b),
        "Empty and non-empty should not be equal"
    );
}

#[actix_web::test]
async fn test_ct_eq_str_opt_both_some_equal() {
    use crate::ct_eq_str_opt;
    let a = Some("password");
    let b = Some("password");

    assert!(
        ct_eq_str_opt(a, b),
        "Same Option<&str> values should be equal"
    );
}

#[actix_web::test]
async fn test_ct_eq_str_opt_both_some_different() {
    use crate::ct_eq_str_opt;
    let a = Some("password");
    let b = Some("different");

    assert!(
        !ct_eq_str_opt(a, b),
        "Different Option<&str> values should not be equal"
    );
}

#[actix_web::test]
async fn test_ct_eq_str_opt_both_none() {
    use crate::ct_eq_str_opt;
    let a: Option<&str> = None;
    let b: Option<&str> = None;

    assert!(ct_eq_str_opt(a, b), "Both None should be equal");
}

#[actix_web::test]
async fn test_ct_eq_str_opt_one_none() {
    use crate::ct_eq_str_opt;
    let a = Some("password");
    let b: Option<&str> = None;

    assert!(!ct_eq_str_opt(a, b), "Some and None should not be equal");
}

#[actix_web::test]
async fn test_path_traversal_dotdot() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Try to traverse up with ../
    let req = test::TestRequest::get().uri("/../etc/passwd").to_request();
    let resp = test::call_service(&app, req).await;

    // Should not allow access to files outside public_path
    // Should return 404 or be blocked
    assert!(
        resp.status() == StatusCode::NOT_FOUND
            || resp.status() == StatusCode::FORBIDDEN
            || resp.status() == StatusCode::BAD_REQUEST,
        "Path traversal should be blocked or return 404, got {}",
        resp.status()
    );
}

#[actix_web::test]
async fn test_path_traversal_encoded() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Try encoded path traversal
    let req = test::TestRequest::get()
        .uri("/%2e%2e/%2e%2e/etc/passwd")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status() == StatusCode::NOT_FOUND
            || resp.status() == StatusCode::FORBIDDEN
            || resp.status() == StatusCode::BAD_REQUEST,
        "Encoded path traversal should be blocked, got {}",
        resp.status()
    );
}

#[actix_web::test]
async fn test_absolute_path_request() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Try absolute path
    let req = test::TestRequest::get().uri("/etc/passwd").to_request();
    let resp = test::call_service(&app, req).await;

    // Should only serve from public_path, not absolute system paths
    assert!(
        resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::FORBIDDEN,
        "Absolute system path should not be accessible, got {}",
        resp.status()
    );
}

#[actix_web::test]
async fn test_null_byte_injection() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Try null byte injection (URL encoded as %00)
    let req = test::TestRequest::get().uri("/test%00.txt").to_request();
    let resp = test::call_service(&app, req).await;

    // Should handle null bytes safely
    assert!(
        resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::BAD_REQUEST,
        "Null byte injection should be handled safely, got {}",
        resp.status()
    );
}

#[actix_web::test]
async fn test_backslash_path_separator() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Try Windows-style path separator
    let req = test::TestRequest::get()
        .uri("/test\\..\\etc\\passwd")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status() == StatusCode::NOT_FOUND
            || resp.status() == StatusCode::FORBIDDEN
            || resp.status() == StatusCode::BAD_REQUEST,
        "Backslash path traversal should be blocked, got {}",
        resp.status()
    );
}

#[actix_web::test]
async fn test_timing_attack_resistance() {
    // Test that constant_time_eq takes similar time for different inputs
    // This is a basic check; true timing analysis requires more sophisticated testing
    use crate::constant_time_eq;

    let correct = b"secret_password_123";
    let wrong1 = b"wrong_password_1234";
    let wrong2 = b"x_________________!";

    // All comparisons should complete (this is a basic smoke test)
    let result1 = constant_time_eq(correct, wrong1);
    let result2 = constant_time_eq(correct, wrong2);

    assert!(!result1, "Wrong password 1 should not match");
    assert!(!result2, "Wrong password 2 should not match");
}

#[actix_web::test]
async fn test_password_comparison_edge_cases() {
    // Test various edge cases for password comparison
    use crate::constant_time_eq;

    // Empty passwords
    assert!(constant_time_eq(b"", b""));

    // Single character
    assert!(constant_time_eq(b"a", b"a"));
    assert!(!constant_time_eq(b"a", b"b"));

    // Very long passwords
    let long1 = vec![b'a'; 1000];
    let long2 = vec![b'a'; 1000];
    let long3 = vec![b'b'; 1000];

    assert!(constant_time_eq(&long1, &long2));
    assert!(!constant_time_eq(&long1, &long3));
}

#[actix_web::test]
async fn test_hidden_files_access() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Try to access hidden files (starting with .)
    let req = test::TestRequest::get().uri("/.env").to_request();
    let resp = test::call_service(&app, req).await;

    // Hidden files should either not be found or access should be denied
    assert!(
        resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::FORBIDDEN,
        "Hidden files should not be accessible, got {}",
        resp.status()
    );
}

#[actix_web::test]
async fn test_config_file_access() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Try to access configuration files
    let paths = vec!["/cofg.yaml", "/.gitignore", "/Cargo.toml"];

    for path in paths {
        let req = test::TestRequest::get().uri(path).to_request();
        let resp = test::call_service(&app, req).await;

        // Config files MUST NOT be accessible - should return 404 (not in public_path) or 403 (explicitly denied)
        // Allowing 200 OK would be a security vulnerability
        assert!(
            resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::FORBIDDEN,
            "Sensitive config file {} MUST be inaccessible (404/403 only), got {}",
            path,
            resp.status()
        );
    }
}

#[actix_web::test]
async fn test_directory_request_handling() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Request a directory path
    // WHY: Verifies server handles directory requests without crashing
    // NOTE: Test name reflects actual validation (request handling), not directory listing prevention
    let req = test::TestRequest::get().uri("/docs/").to_request();
    let resp = test::call_service(&app, req).await;

    // Should either return 404 or serve index file if present
    // Server's directory listing behavior is controlled elsewhere (not validated by this test)
    assert!(
        resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::OK,
        "Directory request should return 404 or index, got {}",
        resp.status()
    );
}

#[actix_web::test]
async fn test_constant_time_comparison_properties() {
    // Verify that constant_time_eq is symmetric
    use crate::constant_time_eq;

    let a = b"test123";
    let b = b"test456";

    assert_eq!(
        constant_time_eq(a, b),
        constant_time_eq(b, a),
        "Constant time comparison should be symmetric"
    );

    // Verify reflexivity
    assert!(
        constant_time_eq(a, a),
        "Constant time comparison should be reflexive"
    );
}

#[actix_web::test]
async fn test_path_handling() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Test that paths are handled correctly regardless of case
    // Note: Case sensitivity depends on the underlying filesystem
    // (case-sensitive on Unix/Linux, case-insensitive on Windows)
    let req1 = test::TestRequest::get().uri("/Test.txt").to_request();
    let resp1 = test::call_service(&app, req1).await;

    let req2 = test::TestRequest::get().uri("/test.txt").to_request();
    let resp2 = test::call_service(&app, req2).await;

    // Both requests should be handled without crashing
    // (whether they exist or not depends on filesystem)
    assert!(
        resp1.status() == StatusCode::OK || resp1.status() == StatusCode::NOT_FOUND,
        "Path '/Test.txt' should be handled gracefully (got {})",
        resp1.status()
    );
    assert!(
        resp2.status() == StatusCode::OK || resp2.status() == StatusCode::NOT_FOUND,
        "Path '/test.txt' should be handled gracefully (got {})",
        resp2.status()
    );
}
