use actix_web::{ test, web, App, HttpRequest, HttpResponse };

use crate::{ http_ext::HttpRequestCachedExt, Cofg };

#[actix_web::test]
async fn cached_helpers_are_stable() {
  let app = test::init_service(
    App::new().route(
      "/{filename:.*}",
      web::get().to(|req: HttpRequest| async move {
        // touch multiple times to exercise cache
        let _d1 = req.cached_decoded_uri();
        let _d2 = req.cached_decoded_uri();

        let c = Cofg::new();
        let f1 = req.cached_filename_path();
        let f2 = req.cached_filename_path();
        assert_eq!(f1, f2);

        let p1 = req.cached_public_req_path(&c);
        let p2 = req.cached_public_req_path(&c);
        assert_eq!(p1, p2);

        let m1 = req.cached_is_markdown(&c);
        let m2 = req.cached_is_markdown(&c);
        assert_eq!(m1, m2);

        Ok::<_, actix_web::Error>(HttpResponse::Ok().finish())
      })
    )
  ).await;

  let req = test::TestRequest::get().uri("/foo/bar.md").to_request();
  let _ = test::call_service(&app, req).await;
}
