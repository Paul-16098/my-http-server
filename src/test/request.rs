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
use build_fs_tree::Build;

// Note: server_error function is primarily exercised via request handlers that return errors.
// A dedicated integration test (test_server_error_function in src/test/integration.rs) validates it directly.

#[actix_web::test]
async fn test_root_path_request() {
	crate::test::support::init_test_setup();
	crate::test::support::init_test_dir()(build_fs_tree::dir! {})
		.build(crate::test::support::PUBLIC_DIR.get().unwrap())
		.unwrap();

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
	crate::test::support::init_test_dir()(build_fs_tree::dir! {
		"file.with.multiple.dots.txt" =>  build_fs_tree::file!("Content of the file with multiple dots."),
	})
	.build(crate::test::support::PUBLIC_DIR.get().unwrap())
	.unwrap();

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

#[actix_web::test]
async fn test_accept_markdown() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get()
		.uri("/dir/test.md")
		.insert_header(("accept", "text/markdown"))
		.to_request();

	let resp = test::call_service(&app, req).await;
	let body = resp.response().body();

	insta::assert_debug_snapshot!(body, @r#"
	BoxBody(
	    Stream(
	        "dyn MessageBody",
	    ),
	)
	"#);
}

#[ignore = "this not work on github actions"]
#[actix_web::test]
async fn test_accept_html() {
	crate::test::support::init_test_setup();

	let app = test::init_service(App::new().service(main_req)).await;

	let req = test::TestRequest::get()
		.uri("/dir/test.md")
		.insert_header(("accept", "text/html"))
		.to_request();

	let resp = test::call_service(&app, req).await;
	let body = resp.response().body();

	insta::assert_debug_snapshot!(body, @r#"
	BoxBody(
	    Bytes(
	        b"<!DOCTYPE html>\n<html lang=\"auto\">\n  <head>\n    <meta charset=\"utf-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n\n    <link rel=\"stylesheet\" type=\"text/css\" media=\"screen\"\n      href=\"https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.8.1/github-markdown.min.css\" />\n    <style>\n      body.markdown-body {\n        margin: 1.5em;\n      }\n    </style>\n    <title>&quot;dir\\\\test.md&quot;</title>\n  </head>\n  <body class=\"markdown-body\">\n    <h1>h1</h1><h2>h2</h2><h3>h3</h3><h4>h4</h4><h5>h5</h5><h6>h6</h6><hr></hr><h2>list</h2><ul class=\"markdown-list-kind-dash\"><li><p>a</p></li><li><p>b</p></li><li><p>c</p><ol start=\"1\"><li><p>1</p></li><li><p>2</p></li></ol></li></ul><ol start=\"1\"><li><p>1</p></li><li><p>2</p></li><li><p>3</p></li></ol><pre><code>hw</code></pre><pre><code>diff --git a/README.md b/README.md\nindex 05bd665..3430a5a 100644\n--- a/README.md\n+++ b/README.md\n@@ -1,4 +1,6 @@\n-# use\n+# my-web\n+\n+## use\n\n 1. `pip install -r .\\pyproject.toml` or\n    `uv pip install -r .\\pyproject.toml`</code></pre><blockquote><p>a</p><p>b</p></blockquote><hr></hr><blockquote><blockquote><p>c\nd</p></blockquote></blockquote><hr></hr><div class=\"markdown-alert markdown-alert-note\"><p class=\"markdown-alert-title\"><svg class=\"octicon octicon-info mr-2\" viewBox=\"0 0 16 16\" version=\"1.1\" width=\"16\" height=\"16\" aria-hidden=\"true\"><path d=\"M0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8Zm8-6.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM6.5 7.75A.75.75 0 0 1 7.25 7h1a.75.75 0 0 1 .75.75v2.75h.25a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1 0-1.5h.25v-2h-.25a.75.75 0 0 1-.75-.75ZM8 6a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z\"></path></svg>Note</p><p><code>Highlights</code> information that users should take into account, even when skimming.</p></div><hr></hr><div class=\"markdown-alert markdown-alert-tip\"><p class=\"markdown-alert-title\"><svg class=\"octicon octicon-light-bulb mr-2\" viewBox=\"0 0 16 16\" version=\"1.1\" width=\"16\" height=\"16\" aria-hidden=\"true\"><path d=\"M8 1.5c-2.363 0-4 1.69-4 3.75 0 .984.424 1.625.984 2.304l.214.253c.223.264.47.556.673.848.284.411.537.896.621 1.49a.75.75 0 0 1-1.484.211c-.04-.282-.163-.547-.37-.847a8.456 8.456 0 0 0-.542-.68c-.084-.1-.173-.205-.268-.32C3.201 7.75 2.5 6.766 2.5 5.25 2.5 2.31 4.863 0 8 0s5.5 2.31 5.5 5.25c0 1.516-.701 2.5-1.328 3.259-.095.115-.184.22-.268.319-.207.245-.383.453-.541.681-.208.3-.33.565-.37.847a.751.751 0 0 1-1.485-.212c.084-.593.337-1.078.621-1.489.203-.292.45-.584.673-.848.075-.088.147-.173.213-.253.561-.679.985-1.32.985-2.304 0-2.06-1.637-3.75-4-3.75ZM5.75 12h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1 0-1.5ZM6 15.25a.75.75 0 0 1 .75-.75h2.5a.75.75 0 0 1 0 1.5h-2.5a.75.75 0 0 1-.75-.75Z\"></path></svg>Tip</p><p>Optional information to help a user be more successful.</p></div><hr></hr><div class=\"markdown-alert markdown-alert-important\"><p class=\"markdown-alert-title\"><svg class=\"octicon octicon-report mr-2\" viewBox=\"0 0 16 16\" version=\"1.1\" width=\"16\" height=\"16\" aria-hidden=\"true\"><path d=\"M0 1.75C0 .784.784 0 1.75 0h12.5C15.216 0 16 .784 16 1.75v9.5A1.75 1.75 0 0 1 14.25 13H8.06l-2.573 2.573A1.458 1.458 0 0 1 3 14.543V13H1.75A1.75 1.75 0 0 1 0 11.25Zm1.75-.25a.25.25 0 0 0-.25.25v9.5c0 .138.112.25.25.25h2a.75.75 0 0 1 .75.75v2.19l2.72-2.72a.749.749 0 0 1 .53-.22h6.5a.25.25 0 0 0 .25-.25v-9.5a.25.25 0 0 0-.25-.25Zm7 2.25v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 9a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z\"></path></svg>Important</p><p>Crucial information necessary for users to succeed.</p></div><hr></hr><div class=\"markdown-alert markdown-alert-warning\"><p class=\"markdown-alert-title\"><svg class=\"octicon octicon-alert mr-2\" viewBox=\"0 0 16 16\" version=\"1.1\" width=\"16\" height=\"16\" aria-hidden=\"true\"><path d=\"M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575Zm1.763.707a.25.25 0 0 0-.44 0L1.698 13.132a.25.25 0 0 0 .22.368h12.164a.25.25 0 0 0 .22-.368Zm.53 3.996v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 11a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z\"></path></svg>Warning</p><p>Critical content demanding immediate user attention due to potential risks.</p></div><hr></hr><div class=\"markdown-alert markdown-alert-caution\"><p class=\"markdown-alert-title\"><svg class=\"octicon octicon-stop mr-2\" viewBox=\"0 0 16 16\" version=\"1.1\" width=\"16\" height=\"16\" aria-hidden=\"true\"><path d=\"M4.47.22A.749.749 0 0 1 5 0h6c.199 0 .389.079.53.22l4.25 4.25c.141.14.22.331.22.53v6a.749.749 0 0 1-.22.53l-4.25 4.25A.749.749 0 0 1 11 16H5a.749.749 0 0 1-.53-.22L.22 11.53A.749.749 0 0 1 0 11V5c0-.199.079-.389.22-.53Zm.84 1.28L1.5 5.31v5.38l3.81 3.81h5.38l3.81-3.81V5.31L10.69 1.5ZM8 4a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 8 4Zm0 8a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z\"></path></svg>Caution</p><p>Negative potential consequences of an action.</p></div><hr></hr><p><img src=\"image.jpg\" alt=\"\"></img></p><p><img src=\"image.jpg\" alt=\"has alt\"></img></p>\n\n    <hr />\n    <a href=\"/\">goto root</a>\n    <footer style=\"text-align: center\">\n      <a style=\"color: rgba(0, 0, 0, 0.489)\" href=\"https://github.com/Paul-16098/my-http-server/\">my-http-server\n        v4.1.10</a>\n    </footer>\n  </body>\n</html>",
	    ),
	)
	"#);
}
