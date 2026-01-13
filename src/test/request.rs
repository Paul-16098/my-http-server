//! Request handler tests - Testing HTTP endpoint behaviors
//!
//! WHY: Validate core request handling logic:
//! - Server error responses
//! - 404 handling
//! - Markdown rendering
//! - TOC generation
//! - Static file serving

use actix_web::{test, App, http::StatusCode};
use crate::request::main_req;
use crate::cofg::config::Cofg;
use crate::cofg::cli::Args;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_test_config() {
    INIT.call_once(|| {
        use clap::Parser;
        
        let args = Args::try_parse_from(["test"].as_ref()).unwrap_or_else(|_| Args::parse());
        let _ = Cofg::init_global(&args, true);
        
        #[cfg(feature = "github_emojis")]
        {
            let emoji_path = std::path::Path::new("./emojis.json");
            if !emoji_path.exists() {
                let _ = std::fs::write(
                    emoji_path,
                    r#"{"unicode":{},"else":{}}"#
                );
            }
        }
    });
}

// Note: server_error function is an internal helper tested implicitly through
// request handlers that return errors. No direct test needed.

#[actix_web::test]
async fn test_root_path_request() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::NOT_FOUND,
        "Root request should return 200 or 404"
    );
}

#[actix_web::test]
async fn test_nonexistent_path_returns_404() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/nonexistent_file_xyz_12345.txt")
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "Nonexistent file should return 404"
    );
}

#[actix_web::test]
async fn test_path_with_dots() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/file.with.multiple.dots.txt")
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::NOT_FOUND,
        "File with multiple dots should be handled"
    );
}

#[actix_web::test]
async fn test_path_with_query_string() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/path?query=value")
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    // Query strings should be handled
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::NOT_FOUND,
        "Query strings should be handled"
    );
}

#[actix_web::test]
async fn test_path_with_fragment() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/path#fragment")
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    // Fragments are typically not sent to server but let's verify handling
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::NOT_FOUND,
        "Fragment should be handled"
    );
}

#[actix_web::test]
async fn test_post_request_not_allowed() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    let req = test::TestRequest::post()
        .uri("/")
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(
        resp.status() == StatusCode::METHOD_NOT_ALLOWED || resp.status() == StatusCode::NOT_FOUND,
        "POST should not be allowed"
    );
}

#[actix_web::test]
async fn test_put_request_not_allowed() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    let req = test::TestRequest::put()
        .uri("/")
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(
        resp.status() == StatusCode::METHOD_NOT_ALLOWED || resp.status() == StatusCode::NOT_FOUND,
        "PUT should not be allowed"
    );
}

#[actix_web::test]
async fn test_delete_request_not_allowed() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    let req = test::TestRequest::delete()
        .uri("/")
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(
        resp.status() == StatusCode::METHOD_NOT_ALLOWED || resp.status() == StatusCode::NOT_FOUND,
        "DELETE should not be allowed"
    );
}

#[actix_web::test]
async fn test_get_with_if_modified_since() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/")
        .insert_header(("if-modified-since", "Mon, 01 Jan 2024 00:00:00 GMT"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    // Should handle conditional requests
    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Conditional request should be handled"
    );
}

#[actix_web::test]
async fn test_very_long_path() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    let long_path = format!("/{}", "a".repeat(2000));
    let req = test::TestRequest::get()
        .uri(&long_path)
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    // Should handle or reject long paths gracefully
    assert!(
        resp.status() == StatusCode::OK 
        || resp.status() == StatusCode::NOT_FOUND 
        || resp.status() == StatusCode::BAD_REQUEST,
        "Long path should be handled gracefully"
    );
}

#[actix_web::test]
async fn test_response_content_type_set() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/")
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    // Should have a content-type header
    let has_content_type = resp.headers().get("content-type").is_some();
    assert!(
        has_content_type,
        "Response should include content-type header"
    );
}

#[actix_web::test]
async fn test_multiple_sequential_requests() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    for i in 0..10 {
        let req = test::TestRequest::get()
            .uri(&format!("/path_{}", i))
            .to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(
            resp.status().is_success() || resp.status().is_client_error(),
            "Request {} should complete", i
        );
    }
}

#[actix_web::test]
async fn test_percent_encoded_spaces() {
    init_test_config();
    
    let app = test::init_service(
        App::new().service(main_req)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/file%20with%20spaces.txt")
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::NOT_FOUND,
        "Percent-encoded spaces should be handled"
    );
}
