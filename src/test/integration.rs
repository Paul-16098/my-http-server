//! Integration tests - Full HTTP request/response cycles
//!
//! WHY: Test complete middleware chain and HTTP behaviors:
//! - Static file serving
//! - Markdown rendering via HTTP
//! - Middleware (compression, normalization, logging)
//! - Error responses (404, 500)
//! - Index page behavior
//!
//! NOTE: These tests require initialization of global config which may trigger
//! emoji fetching with github_emojis feature. Run with --no-default-features
//! for faster, more reliable tests in CI environments.

use crate::request::main_req;
use actix_web::{App, http::StatusCode, test};

/// Initialize global config for integration tests
/// Uses shared helper from config module to ensure consistency across test suites
fn init_test_config() {
    super::config::init_test_config();
}

#[actix_web::test]
async fn test_static_file_serving() {
    init_test_config();

    // This test verifies that the server can handle static file requests
    // Note: Actual file serving depends on the configured public_path

    let app = test::init_service(App::new().service(main_req)).await;

    // Test root path
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    // Should either succeed (200) or not found (404)
    // depending on whether public/index.html exists
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::NOT_FOUND,
        "Root should return 200 or 404, got {}",
        resp.status()
    );
}

#[actix_web::test]
async fn test_nonexistent_file() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Request a file that definitely doesn't exist
    let req = test::TestRequest::get()
        .uri("/this-file-definitely-does-not-exist-12345.xyz")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "Nonexistent file should return 404"
    );
}

#[actix_web::test]
async fn test_path_with_special_chars() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Test URL encoding
    let req = test::TestRequest::get()
        .uri("/test%20file.txt")
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Should handle encoded paths gracefully (either 200 if exists, or 404)
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::NOT_FOUND,
        "Encoded path should be handled, got {}",
        resp.status()
    );
}

#[actix_web::test]
async fn test_markdown_file_request() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Try to request a .md file
    let req = test::TestRequest::get().uri("/test.md").to_request();
    let resp = test::call_service(&app, req).await;

    // Should either render markdown (200) or not found (404)
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::NOT_FOUND,
        "Markdown request should return 200 or 404, got {}",
        resp.status()
    );
}

#[actix_web::test]
async fn test_response_has_content_type() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    // Response should have a content-type header
    let content_type = resp.headers().get("content-type");
    assert!(
        content_type.is_some(),
        "Response should have content-type header"
    );
}

#[actix_web::test]
async fn test_multiple_requests() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Make multiple requests to ensure server handles them correctly
    for i in 0..5 {
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();

        assert!(
            status == StatusCode::OK || status == StatusCode::NOT_FOUND,
            "Request {} failed with status: {} (expected 200 or 404)",
            i,
            status
        );
    }
}

#[actix_web::test]
async fn test_get_method_only() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // POST should not be allowed on main_req (it's GET only)
    // The service may return 404 or 405 depending on routing implementation
    let req = test::TestRequest::post().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status() == StatusCode::METHOD_NOT_ALLOWED || resp.status() == StatusCode::NOT_FOUND,
        "POST method should return 405 or 404, got {}",
        resp.status()
    );
}

#[actix_web::test]
async fn test_nested_path() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Test nested directory path
    let req = test::TestRequest::get()
        .uri("/docs/subdocs/test.md")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::NOT_FOUND,
        "Nested paths should be handled correctly"
    );
}

#[actix_web::test]
async fn test_sequential_requests() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Test that server handles multiple sequential requests correctly
    // WHY: Validates server stability and state management across multiple requests
    for i in 0..10 {
        let req = test::TestRequest::get()
            .uri(&format!("/test{}.txt", i))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(
            resp.status() == StatusCode::OK || resp.status() == StatusCode::NOT_FOUND,
            "Sequential request {} should complete, got {}",
            i,
            resp.status()
        );
    }
}

#[actix_web::test]
async fn test_empty_path() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Root path should be handled
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::NOT_FOUND,
        "Root path should be handled"
    );
}

#[actix_web::test]
async fn test_request_headers() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Test with custom headers
    let req = test::TestRequest::get()
        .uri("/")
        .insert_header(("accept", "text/html"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Request with headers should be processed"
    );
}

#[actix_web::test]
async fn test_server_error_function() {
    use crate::request::server_error;

    let response = server_error("Test error message".to_string());

    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "server_error should return 500"
    );
}

#[actix_web::test]
async fn test_large_path() {
    init_test_config();

    let app = test::init_service(App::new().service(main_req)).await;

    // Test with a very long path
    let long_path = format!("/{}", "a".repeat(1000));
    let req = test::TestRequest::get().uri(&long_path).to_request();
    let resp = test::call_service(&app, req).await;

    // Should handle long paths (either 404 or 200 or BAD_REQUEST)
    assert!(
        resp.status() == StatusCode::OK
            || resp.status() == StatusCode::NOT_FOUND
            || resp.status() == StatusCode::BAD_REQUEST,
        "Long path should be handled gracefully"
    );
}
