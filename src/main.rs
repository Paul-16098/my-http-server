//! main

#[cfg(test)]
mod test;

mod cofg;
mod parser;
use crate::cofg::{ cli, config::Cofg };
mod error;
use crate::error::AppResult;
mod http_ext;
mod request;
use crate::request::{ index, main_req };

use actix_web::{ App, HttpServer, dev::Server, http::KeepAlive, middleware };
use clap::Parser as _;
use log::{ debug, error, info, warn };
use std::fs::create_dir_all;
use std::path::Path;

/// Initialize logging & ensure `public_path` directory exists.
///
/// WHY: Keep side-effect setup isolated from `main()`. Directory creation early prevents
/// per-request race to create it lazily. Logger configured with module paths for traceability.
/// 中文：集中初始化，避免每次請求重複檢查；模組路徑便於除錯追蹤。
fn init(c: &Cofg) -> AppResult<()> {
  logger_init();

  create_dir_all(c.public_path.clone())?;
  create_dir_all("./meta")?;
  if !Path::new("./meta/html-t.templating").exists() {
    error!("missing required template: meta/html-t.templating\nuse default");
    std::fs::write("./meta/html-t.templating", include_str!("../meta/html-t.templating"))?;
    // exit and make user re-run to re-init
    std::process::exit(1);
  }
  Ok(())
}
fn logger_init() {
  env_logger
    ::builder()
    .default_format()
    .format_timestamp(None)
    .format_source_path(true)
    .filter_level(log::LevelFilter::Info)
    .parse_default_env()
    .init();
}

/// Construct Actix `HttpServer` with conditional middleware based on config flags.
///
/// WHY: Keeps `main` concise; middleware toggles (path normalize, compress, logger) are applied
/// only when enabled to avoid unnecessary overhead.
/// 中文：依設定按需加掛 middleware，減少未使用功能的開銷。
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
    .bind(addrs)?
    .run();

  Ok(server)
}

#[actix_web::main]
async fn main() -> AppResult<()> {
  let mut s = cofg::build_config_from_cli(Cofg::new(), &cli::Args::parse())?;
  s.public_path = Path::new(&s.public_path)
    .canonicalize()
    .unwrap_or_else(|e| {
      warn!("Failed to canonicalize public_path '{}': {}", &s.public_path, e);
      (&s.public_path).into()
    })
    .to_string_lossy()
    .to_string();

  init(&s)?;
  info!("VERSION: {}", option_env!("VERSION").unwrap_or("?"));
  debug!("cofg: {s:#?}");

  build_server(&s)?.await?;
  Ok(())
}
