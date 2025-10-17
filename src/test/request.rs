use actix_web::{ body::to_bytes, http::{ header::CONTENT_TYPE, StatusCode }, test, App };

use crate::request::{ main_req, server_error };

// Integration tests covering the main routing behavior

#[actix_web::test]
async fn get_root_renders_toc_when_index_missing() {
  let app = test::init_service(App::new().service(main_req).service(main_req)).await;

  let req = test::TestRequest::get().uri("/").to_request();
  let resp = test::call_service(&app, req).await;
  assert!(resp.status().is_success());

  let body = test::call_and_read_body(&app, test::TestRequest::get().uri("/").to_request()).await;
  let body = String::from_utf8_lossy(&body);
  // Uses template shell and sets path to `toc:index`
  assert!(body.contains("<html"));
  assert!(body.contains("<title>toc:index</title>"));
}

#[actix_web::test]
async fn get_markdown_file_is_rendered() {
  let app = test::init_service(App::new().service(main_req).service(main_req)).await;

  let req = test::TestRequest::get().uri("/dir/test.md").to_request();
  let resp = test::call_service(&app, req).await;
  assert!(resp.status().is_success());

  let body = test::call_and_read_body(
    &app,
    test::TestRequest::get().uri("/dir/test.md").to_request()
  ).await;
  let body = String::from_utf8_lossy(&body);
  // Rendered markdown should include an h1 and the page title becomes relative path
  assert!(body.contains("<h1>h1</h1>"));
  // Windows may render `dir\test.md` because `.display()` is used; accept both
  let has_title_fwd = body.contains("<title>dir/test.md</title>");
  let has_title_back = body.contains("<title>dir\\test.md</title>");
  assert!(has_title_fwd || has_title_back);
}

#[actix_web::test]
async fn get_static_file_is_streamed() {
  let app = test::init_service(App::new().service(main_req).service(main_req)).await;

  let req = test::TestRequest::get().uri("/dir/test.txt").to_request();
  let resp = test::call_service(&app, req).await;
  assert!(resp.status().is_success());

  let body = test::call_and_read_body(
    &app,
    test::TestRequest::get().uri("/dir/test.txt").to_request()
  ).await;
  let s = String::from_utf8_lossy(&body);
  assert!(s.contains("test1"));
}

#[actix_web::test]
async fn get_directory_renders_toc_page() {
  let app = test::init_service(App::new().service(main_req).service(main_req)).await;

  let req = test::TestRequest::get().uri("/dir/").to_request();
  let resp = test::call_service(&app, req).await;
  assert!(resp.status().is_success());

  let body = test::call_and_read_body(
    &app,
    test::TestRequest::get().uri("/dir/").to_request()
  ).await;
  let body = String::from_utf8_lossy(&body);
  // Directory TOC gets path:toc:<rel-dir> as title and should link to contained markdown
  assert!(body.contains("<title>toc:dir</title>"));
  assert!(body.contains(r#"href="dir/test%2Emd""#));
}

#[actix_web::test]
async fn not_found_uses_custom_404_if_present() {
  let app = test::init_service(App::new().service(main_req).service(main_req)).await;

  let req = test::TestRequest::get().uri("/no-such-file").to_request();
  let resp = test::call_service(&app, req).await;
  assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);

  let body = test::call_and_read_body(
    &app,
    test::TestRequest::get().uri("/no-such-file").to_request()
  ).await;
  let body = String::from_utf8_lossy(&body);
  assert!(body.contains("<h1>404</h1>"));
}

#[actix_web::test]
async fn server_error_sets_status_and_content_type_and_body() {
  let err = "unit test error message".to_string();
  let resp = server_error(err.clone());

  // 狀態碼為 500
  assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

  // Content-Type 為 text/plain（允許含 charset）
  let headers = resp.headers().clone();
  let ct = headers.get(CONTENT_TYPE).expect("missing content-type");
  let ct_str = ct.to_str().expect("invalid content-type header");
  assert!(ct_str.starts_with("text/plain"), "unexpected content-type: {ct_str}");

  // 本文等於輸入字串
  let bytes = to_bytes(resp.into_body()).await.expect("to_bytes failed");
  assert_eq!(bytes, err.as_bytes());
}
