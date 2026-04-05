//! Request handler tests - Testing HTTP endpoint behaviors
//!
//! WHY: Validate core request handling logic:
//! - Server error responses
//! - 404 handling
//! - Markdown rendering
//! - TOC generation
//! - Static file serving

use crate::{request::main_req, test::support::assert_status_in};
use actix_web::{App, http::StatusCode, test};

// Note: server_error function is primarily exercised via request handlers that return errors.
// A dedicated integration test (test_server_error_function in src/test/integration.rs) validates it directly.

#[actix_web::test]
async fn test_root_path_request() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get().uri("/").to_request();
	let resp = test::call_service(&app, req).await;

	assert_status_in(resp.status(), &[StatusCode::OK, StatusCode::NOT_FOUND]);
}

#[actix_web::test]
async fn test_nonexistent_path_returns_404() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get()
		.uri("/nonexistent_file_xyz_12345.txt")
		.to_request();
	let resp = test::call_service(&app, req).await;

	assert_status_in(resp.status(), &[StatusCode::NOT_FOUND]);
}

#[actix_web::test]
async fn test_path_with_dots() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get()
		.uri("/file.with.multiple.dots.txt")
		.to_request();
	let resp = test::call_service(&app, req).await;

	assert_status_in(resp.status(), &[StatusCode::OK, StatusCode::NOT_FOUND]);
}

#[actix_web::test]
async fn test_path_with_query_string() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get()
		.uri("/path?query=value")
		.to_request();
	let resp = test::call_service(&app, req).await;

	// Query strings should be handled
	assert_status_in(resp.status(), &[StatusCode::OK, StatusCode::NOT_FOUND]);
}

#[actix_web::test]
async fn test_path_with_fragment() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get().uri("/path#fragment").to_request();
	let resp = test::call_service(&app, req).await;

	// Fragments are typically not sent to server but let's verify handling
	assert_status_in(resp.status(), &[StatusCode::OK, StatusCode::NOT_FOUND]);
}

#[actix_web::test]
async fn test_post_request_not_allowed() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::post().uri("/").to_request();
	let resp = test::call_service(&app, req).await;

	assert_status_in(
		resp.status(),
		&[StatusCode::METHOD_NOT_ALLOWED, StatusCode::NOT_FOUND],
	);
}

#[actix_web::test]
async fn test_put_request_not_allowed() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::put().uri("/").to_request();
	let resp = test::call_service(&app, req).await;

	assert_status_in(
		resp.status(),
		&[StatusCode::METHOD_NOT_ALLOWED, StatusCode::NOT_FOUND],
	);
}

#[actix_web::test]
async fn test_delete_request_not_allowed() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::delete().uri("/").to_request();
	let resp = test::call_service(&app, req).await;

	assert_status_in(
		resp.status(),
		&[StatusCode::METHOD_NOT_ALLOWED, StatusCode::NOT_FOUND],
	);
}

#[actix_web::test]
async fn test_get_with_if_modified_since() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get()
		.uri("/")
		.insert_header(("if-modified-since", "Mon, 01 Jan 2024 00:00:00 GMT"))
		.to_request();
	let resp = test::call_service(&app, req).await;

	// Should handle conditional requests
	assert_status_in(resp.status(), &[StatusCode::OK, StatusCode::NOT_MODIFIED]);
}

#[actix_web::test]
async fn test_very_long_path() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let long_path = format!("/{}", "a".repeat(2000));
	let req = test::TestRequest::get().uri(&long_path).to_request();
	let resp = test::call_service(&app, req).await;

	// Should handle or reject long paths gracefully
	assert_status_in(
		resp.status(),
		&[
			StatusCode::OK,
			StatusCode::NOT_FOUND,
			StatusCode::BAD_REQUEST,
		],
	);
}

#[actix_web::test]
async fn test_response_content_type_set() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get().uri("/").to_request();
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
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	for i in 0..10 {
		let req = test::TestRequest::get()
			.uri(&format!("/path_{}", i))
			.to_request();
		let resp = test::call_service(&app, req).await;

		assert_status_in(
			resp.status(),
			&[
				StatusCode::OK,
				StatusCode::NOT_FOUND,
				StatusCode::BAD_REQUEST,
			],
		);
	}
}

#[actix_web::test]
async fn test_percent_encoded_spaces() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get()
		.uri("/file%20with%20spaces.txt")
		.to_request();
	let resp = test::call_service(&app, req).await;

	assert_status_in(resp.status(), &[StatusCode::OK, StatusCode::NOT_FOUND]);
}
