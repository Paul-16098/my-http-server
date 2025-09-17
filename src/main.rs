//! main

#[cfg(test)]
mod test;

mod parser;
use crate::parser::markdown::get_toc;
mod cofg;
use crate::cofg::Cofg;
mod error;
use crate::error::{ AppResult, AppError };
use crate::parser::md2html;

use actix_files::NamedFile;
use log::{ debug, error, info, warn };
use std::fs::{ create_dir_all, read_to_string };
use std::path::Path;
use actix_web::{ dev::Server, http::KeepAlive, middleware, App, HttpServer };

fn init() -> AppResult<()> {
  env_logger
    ::builder()
    .default_format()
    .format_timestamp(None)
    .format_module_path(true)
    .filter_level(log::LevelFilter::Info)
    .parse_default_env()
    .init();

  create_dir_all(cofg::Cofg::get(false).public_path)?;
  Ok(())
}

#[actix_web::get("/{filename:.*}")]
async fn main_req(req: actix_web::HttpRequest) -> impl actix_web::Responder {
  use actix_web::{ HttpResponseBuilder, http::StatusCode };

  debug!("{req:#?}");

  let req_path = Path::new(&Cofg::new().public_path).join(
    req.match_info().query("filename").parse::<std::path::PathBuf>().unwrap()
  );
  if !req_path.exists() {
    debug!("{}:!exists", req_path.display());
    return match actix_files::NamedFile::open_async("./meta/404.html").await {
      Ok(file) => {
        let mut res = file.into_response(&req);
        *res.status_mut() = StatusCode::NOT_FOUND;
        res
      }
      Err(e) => {
        warn!("failed to open 404.html: {e}");
        HttpResponseBuilder::new(StatusCode::NOT_FOUND).body("404 Not Found")
      }
    };
  }

  if req_path.extension().and_then(|v| v.to_str()) == Some("md") {
    debug!("is md");
    match read_to_string(&req_path) {
      Ok(file) => {
        let out = crate::parser::md2html(
          file,
          &crate::Cofg::new(),
          vec![format!("path:{}", req_path.display())]
        );
        match out {
          Ok(html) => HttpResponseBuilder::new(StatusCode::OK).body(html),
          Err(err) => {
            warn!("{err}");
            HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).body(err.to_string())
          }
        }
      }
      Err(err) => {
        warn!("{err}");
        HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).body(err.to_string())
      }
    }
  } else {
    debug!("no md");
    match NamedFile::open_async(req_path).await {
      Ok(file) => file.into_response(&req),
      Err(err) => {
        warn!("{err}");
        HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).body(err.to_string())
      }
    }
  }
}

#[actix_web::get("/", name = "index")]
async fn index(_: actix_web::HttpRequest) -> impl actix_web::Responder {
  use actix_web::HttpResponseBuilder;

  let c = &Cofg::new();

  let index_file = Path::new(&c.public_path).join("index.html");
  if index_file.exists() {
    let f = read_to_string(index_file);
    if let Ok(value) = f {
      debug!("index exitis=>show file");
      HttpResponseBuilder::new(actix_web::http::StatusCode::OK).body(value)
    } else {
      let err = f.err().unwrap();
      warn!("{err}");
      HttpResponseBuilder::new(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR).body(
        err.to_string()
      )
    }
  } else {
    debug!("index !exitis=>get toc");
    let toc = get_toc(c);
    if let Ok(v) = toc {
      let r = md2html(v, c, vec!["path:toc".to_string()]);
      if let Ok(html) = r {
        HttpResponseBuilder::new(actix_web::http::StatusCode::OK).body(html)
      } else {
        let err = r.err().unwrap();
        warn!("{err}");
        HttpResponseBuilder::new(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR).body(
          err.to_string()
        )
      }
    } else {
      let err = toc.err().unwrap();
      warn!("{err}");
      HttpResponseBuilder::new(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR).body(
        err.to_string()
      )
    }
  }
}

fn build_server(s: &Cofg) -> std::io::Result<Server> {
  let middleware_cofg = s.middleware.clone();
  let addrs = &s.addrs;
  info!("run in http://{}/", addrs);

  let server = HttpServer::new(move || {
    App::new()
      .wrap(
        middleware::Condition::new(
          middleware_cofg.normalize_path,
          middleware::NormalizePath::new(middleware::TrailingSlash::Trim)
        )
      )
      .wrap(middleware::Condition::new(middleware_cofg.compress, middleware::Compress::default()))
      .wrap(
        middleware::Condition::new(
          middleware_cofg.logger.enabling,
          middleware::Logger
            ::new(&middleware_cofg.logger.format)
            .custom_request_replace("url", |req| {
              let u = &req.uri().to_string();
              let mut u = percent_encoding
                ::percent_decode(u.as_bytes())
                .decode_utf8()
                .unwrap_or(std::borrow::Cow::Borrowed(u))
                .to_string();
              if u.starts_with('/') {
                u.remove(0);
              }
              u
            })
        )
      )
      .service(index)
      .service(main_req)
  })
    .keep_alive(KeepAlive::Os)
    .bind(addrs.to_string())?
    .run();

  Ok(server)
}

#[actix_web::main]
async fn main() -> Result<(), AppError> {
  let s = cofg::Cofg::get(false);
  init()?;
  debug!("cofg: {s:#?}");
  // md2html_all()?;
  // if s.toc.make_toc {
  //   make_toc()?;
  // }

  build_server(&s).unwrap().await.unwrap();
  Ok(())
}
