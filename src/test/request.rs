use actix_web::{ test, App };

use crate::request::{ index, main_req };

// Integration tests covering the main routing behavior

#[actix_web::test]
async fn get_root_renders_toc_when_index_missing() {
  let app = test::init_service(App::new().service(index).service(main_req)).await;

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
  let app = test::init_service(App::new().service(index).service(main_req)).await;

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
  let app = test::init_service(App::new().service(index).service(main_req)).await;

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
  let app = test::init_service(App::new().service(index).service(main_req)).await;

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
  let app = test::init_service(App::new().service(index).service(main_req)).await;

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
