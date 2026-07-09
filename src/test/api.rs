use actix_web::{App, http::StatusCode, test};

use crate::test::support::assert_status_in;

#[actix_web::test]
async fn test_api_get_raw() {
	crate::test::support::init_test_setup();

	let test_body = "# Hello\n";

	crate::test::support::init_public_dir(build_fs_tree::dir! {
		"dir" => build_fs_tree::dir! {
			"test.md" => build_fs_tree::file!(test_body),
		}
	});

	let app = test::init_service(App::new().service(crate::api::service())).await;

	let req = test::TestRequest::post()
		.uri("/api/file/get_raw")
		.set_payload("dir/test.md")
		.to_request();

	let resp = test::call_service(&app, req).await;

	assert_status_in(resp.status(), &[StatusCode::OK]);

	let body = test::read_body(resp).await;

	assert_eq!(body, test_body);
}

#[actix_web::test]
async fn test_api_check_exists() {
	crate::test::support::init_test_setup();

	crate::test::support::init_public_dir(build_fs_tree::dir! {
		"dir" => build_fs_tree::dir! {
			"test.md" => build_fs_tree::file!("test_body"),
		}
	});

	let app = test::init_service(App::new().service(crate::api::service())).await;

	let req_1 = test::TestRequest::post()
		.uri("/api/file/exists")
		.set_payload("dir/test.md")
		.to_request();

	let resp_1 = test::call_service(&app, req_1).await;

	assert_status_in(resp_1.status(), &[StatusCode::OK]);

	let body_1: crate::api::file::ExistsResponse = test::read_body_json(resp_1).await;

	assert_eq!(
		body_1,
		crate::api::file::ExistsResponse {
			exists: true,
			path_type: Some(crate::api::file::PathType::File)
		}
	);

	let req_2 = test::TestRequest::post()
		.uri("/api/file/exists")
		.set_payload("dir/non-existent.md")
		.to_request();

	let resp_2 = test::call_service(&app, req_2).await;

	assert_status_in(resp_2.status(), &[StatusCode::OK]);

	let body_2: crate::api::file::ExistsResponse = test::read_body_json(resp_2).await;

	assert_eq!(
		body_2,
		crate::api::file::ExistsResponse {
			exists: false,
			path_type: None
		}
	);
}
