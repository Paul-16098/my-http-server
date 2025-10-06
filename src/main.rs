//! main

#[cfg(test)]
mod test;

mod parser;
mod cofg;
use crate::cofg::{ Cofg, CofgAddrs, cli };
mod error;
use crate::error::{ AppResult, AppError };
mod http_ext;
mod request;
use crate::request::{ index, main_req };

use clap::Parser;
use log::{ debug, error, info };
use std::fs::create_dir_all;
use std::path::Path;
use actix_web::{ dev::Server, http::KeepAlive, middleware, App, HttpServer };

/// Initialize logging & ensure `public_path` directory exists.
///
/// WHY: Keep side-effect setup isolated from `main()`. Directory creation early prevents
/// per-request race to create it lazily. Logger configured with module paths for traceability.
/// 中文：集中初始化，避免每次請求重複檢查；模組路徑便於除錯追蹤。
fn init(c: &Cofg) -> AppResult<()> {
  env_logger
    ::builder()
    .default_format()
    .format_timestamp(None)
    .format_module_path(true)
    .filter_level(log::LevelFilter::Info)
    .parse_default_env()
    .init();

  create_dir_all(c.public_path.clone())?;
  Ok(())
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
    .bind(addrs.to_string())?
    .run();

  Ok(server)
}

/// Merge CLI overrides into loaded config.
///
/// WHY: Preserve file-based config as baseline; explicit CLI flags have higher precedence.
/// 中文：以設定檔為基礎，命令列參數覆寫對應欄位。
fn build_config_from_cli(mut s: Cofg, cli: &cli::Args) -> Cofg {
  match (&cli.ip, cli.port) {
    (None, None) => (),
    (None, Some(port)) => {
      s.addrs.port = port;
    }
    (Some(ip), None) => {
      s.addrs.ip = ip.to_string();
    }
    (Some(_), Some(_)) => {
      s.addrs = CofgAddrs::from(cli);
    }
  }
  s
}

#[actix_web::main]
async fn main() -> Result<(), AppError> {
  let s = build_config_from_cli(Cofg::new(), &cli::Args::parse());

  init(&s)?;
  info!("VERSION: {}", option_env!("VERSION").unwrap_or("?"));
  debug!("cofg: {s:#?}");

  if !Path::new("./meta/html-t.templating").exists() {
    error!("missing required template: meta/html-t.templating\nuse default");
    std::fs::write("./meta/html-t.templating", include_str!("../docker/meta/html-t.templating"))?;
    // exit and make user re-run to re-init
    std::process::exit(1);
  }
  build_server(&s)?.await?;
  Ok(())
}
