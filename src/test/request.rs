//! Request handler tests - Testing HTTP endpoint behaviors
//!
//! WHY: Validate core request handling logic:
//! - Server error responses
//! - 404 handling
//! - Markdown rendering
//! - TOC generation
//! - Static file serving

use crate::{request::main_req, test::support::assert_status_in};
use actix_web::{
	App,
	http::{StatusCode, header},
	test,
};

// Note: server_error function is primarily exercised via request handlers that return errors.
// A dedicated integration test (test_server_error_function in src/test/integration.rs) validates it directly.

#[actix_web::test]
async fn test_root_path_request() {
	crate::test::support::init_test_setup();
	crate::test::support::init_public_dir(build_fs_tree::dir! {});

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get().uri("/").to_request();
	let resp = test::call_service(&app, req).await;

	assert_status_in(resp.status(), &[StatusCode::OK]);
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
	crate::test::support::init_public_dir(build_fs_tree::dir! {
		"file.with.multiple.dots.txt" =>  build_fs_tree::file!("Content of the file with multiple dots."),
	});

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get()
		.uri("/file.with.multiple.dots.txt")
		.to_request();
	let resp = test::call_service(&app, req).await;

	assert_status_in(resp.status(), &[StatusCode::OK]);
}

#[actix_web::test]
async fn test_path_with_query_string() {
	crate::test::support::init_test_setup();

	crate::test::support::init_public_dir(build_fs_tree::dir! {
			"path" => build_fs_tree::file!("test_body")
	});

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

	crate::test::support::init_public_dir(build_fs_tree::dir! {
			"path" => build_fs_tree::file!("test_body")
	});

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get().uri("/path#fragment").to_request();
	let resp = test::call_service(&app, req).await;

	// Fragments are typically not sent to server but let's verify handling
	assert_status_in(resp.status(), &[StatusCode::OK]);
}

#[actix_web::test]
async fn test_very_long_path() {
	crate::test::support::init_test_setup();

	crate::test::support::init_public_dir(build_fs_tree::dir! {
			"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" => build_fs_tree::file!("test_body")
	});

	let app = test::init_service(App::new().service(main_req)).await;

	let long_path = format!("/{}", "a".repeat(100));
	let req = test::TestRequest::get().uri(&long_path).to_request();
	let resp = test::call_service(&app, req).await;

	// Should handle or reject long paths gracefully
	assert_status_in(resp.status(), &[StatusCode::OK]);
}

#[actix_web::test]
async fn test_response_content_type_set() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get().uri("/").to_request();
	let resp = test::call_service(&app, req).await;

	// Should have a content-type header
	let content_type = resp.headers().get("content-type");
	assert_eq!(
		content_type,
		Some(&header::HeaderValue::from_static(
			"text/html; charset=utf-8"
		))
	);
}

#[actix_web::test]
async fn test_percent_encoded_spaces() {
	crate::test::support::init_test_setup();

	crate::test::support::init_public_dir(build_fs_tree::dir! {
		"file with spaces.txt" => build_fs_tree::file!("test_body")
	});

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get()
		.uri("/file%20with%20spaces.txt")
		.to_request();
	let resp = test::call_service(&app, req).await;

	assert_status_in(resp.status(), &[StatusCode::OK]);
}

#[actix_web::test]
async fn test_accept_markdown() {
	crate::test::support::init_test_setup();

	let test_body = "# Hello\n";

	crate::test::support::init_public_dir(build_fs_tree::dir! {
		"dir" => build_fs_tree::dir! {
			"test.md" => build_fs_tree::file!(test_body),
		}
	});

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get()
		.uri("/dir/test.md")
		.insert_header(("accept", "text/markdown"))
		.to_request();

	let resp = test::call_service(&app, req).await;

	assert_status_in(resp.status(), &[StatusCode::OK]);

	let body = test::read_body(resp).await;

	assert_eq!(body, test_body);
}
