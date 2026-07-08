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
use log::debug;
use std::sync::Once;

use crate::cofg::{cli, config::Cofg};

/// Initialize logger exactly once per test process using thread-safe guard
///
/// WHY: `env_logger::init()` panics if called multiple times in the same process.
/// Tests running in parallel would trigger this panic without a guard.
/// Using `try_init()` with `Once` ensures logger is set up only once.
pub(crate) fn init_logger_once() {
	static INIT: Once = Once::new();
	INIT.call_once(|| {
		let _ = env_logger::builder()
			.filter_level(log::LevelFilter::Trace)
			.is_test(true)
			.try_init();
	});
}

/// Combined initialization for logger and config
///
/// WHY: Most tests need both logger and config initialized.
/// This helper reduces two calls to one.
pub(crate) fn init_test_setup() {
	init_logger_once();
	init_test_config();
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

pub(crate) static PUBLIC_DIR: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub(crate) fn init_test_dir() -> fn(
	_: build_fs_tree::FileSystemTree<&'static str, &'static str>,
) -> build_fs_tree::FileSystemTree<&'static str, &'static str> {
	build_fs_tree::FileSystemTree::<&'static str, &'static str>::from
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

		PUBLIC_DIR.get_or_init(|| {
			// Create a temporary public directory for tests
			tempfile::TempDir::with_prefix("my-http-server-test-public-")
				.expect("Failed to create temp dir")
				.path()
				.to_string_lossy()
				.to_string()
		});

		let args = cli::Args::try_parse_from(["--public_path", PUBLIC_DIR.get().unwrap()].as_ref())
			.unwrap_or_else(|_| cli::Args::parse());
		debug!("init_test_config: args={:?}", args);
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

/// Helper function to create a temporary directory for test fixtures
pub(crate) fn create_test_dir() -> tempfile::TempDir {
	tempfile::TempDir::with_prefix("my-http-server-test-").expect("Failed to create temp dir")
}
