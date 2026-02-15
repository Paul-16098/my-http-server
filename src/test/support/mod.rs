//! Test Support Harness - Unified utilities to reduce boilerplate across test suites
//!
//! WHY: Consolidate repeated test setup patterns (logger, config, assertions)
//! into reusable helpers. Prevents:
//! - Logger re-initialization panics when tests run in parallel
//! - Copy-pasted initialization code across test files
//! - Inconsistent test setup patterns
//!
//! USAGE:
//! ```rust
//! use crate::test::support::*;
//!
//! #[actix_web::test]
//! async fn test_example() {
//!     init_test_setup();
//!     // Use actix-web test utilities directly
//!     let app = test::init_service(App::new().service(main_req)).await;
//!     let req = test::TestRequest::get().uri("/").to_request();
//!     let resp = test::call_service(&app, req).await;
//!     assert_status_in(resp.status(), &[StatusCode::OK, StatusCode::NOT_FOUND]);
//! }
//! ```

use actix_web::http::StatusCode;
use std::sync::Once;

/// Initialize logger exactly once per test process using thread-safe guard
///
/// WHY: `env_logger::init()` panics if called multiple times in the same process.
/// Tests running in parallel would trigger this panic without a guard.
/// Using `try_init()` with `Once` ensures logger is set up only once.
pub(crate) fn init_logger_once() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
    });
}

/// Initialize global config exactly once per test process using thread-safe guard
///
/// WHY: Config initialization involves:
/// - File I/O for XDG directories (skipped in tests with `no_xdg=true`)
/// - Potential GitHub API calls for emoji cache (with `github_emojis` feature)
/// - One-time setup that should not repeat across parallel tests
pub(crate) fn init_global_config_once() {
    crate::test::config::init_test_config();
}

/// Combined initialization for logger and config
///
/// WHY: Most tests need both logger and config initialized.
/// This helper reduces two calls to one.
pub(crate) fn init_test_setup() {
    init_logger_once();
    init_global_config_once();
}

/// Assert that status code is one of the allowed values
///
/// WHY: Many tests check "status is OK or NOT_FOUND" or similar patterns.
/// This helper makes such assertions more readable and provides better error messages.
pub(crate) fn assert_status_in(status: StatusCode, allowed: &[StatusCode]) {
    assert!(
        allowed.contains(&status),
        "Expected status to be one of {:?}, but got {}",
        allowed,
        status
    );
}
