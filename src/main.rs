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
  if !Path::new("./meta/html-t.hbs").exists() {
    error!("missing required template: meta/html-t.hbs\nuse default");
    std::fs::write("./meta/html-t.hbs", include_str!("../meta/html-t.hbs"))?;
    // exit and make user re-run to re-init
    std::process::exit(1);
  }
  Ok(())
}
fn logger_init() {
  let mut l = env_logger::builder();
  l.default_format()
    .format_timestamp(None)
    .filter_level(log::LevelFilter::Info)
    .parse_default_env();

  #[cfg(debug_assertions)]
  l.format_source_path(true);

  l.init();
}

// SECURITY: 常數時間比較，減少密碼比對的時序攻擊面。
/// 比較長度差與逐位 XOR，避免資料相等時提早返回造成時間差異。
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
  let max_len = a.len().max(b.len());
  let mut diff: u8 = (a.len() ^ b.len()) as u8;
  for i in 0..max_len {
    let ai = *a.get(i).unwrap_or(&0);
    let bi = *b.get(i).unwrap_or(&0);
    diff |= ai ^ bi;
  }
  diff == 0
}

// 對 Option<&str> 進行常數時間比較；只有兩者皆為 Some 時才進行常數時間比較，否則直接返回 false（或 true 若皆為 None）。
fn ct_eq_str_opt(a: Option<&str>, b: Option<&str>) -> bool {
  match (a, b) {
    (Some(a), Some(b)) => constant_time_eq(a.as_bytes(), b.as_bytes()),
    (None, None) => true,
    _ => false,
  }
}

/// Load TLS configuration from certificate and key files.
///
/// WHY: Encapsulate TLS setup logic; read PEM files and construct rustls ServerConfig.
/// 中文：封裝 TLS 設定邏輯，載入憑證與私鑰建立 rustls 設定。
fn load_tls_config(cert_path: &str, key_path: &str) -> AppResult<rustls::ServerConfig> {
  use rustls::pki_types::{ CertificateDer, pem::PemObject as _, PrivateKeyDer };
  use rustls_pki_types::PrivatePkcs8KeyDer;
  use std::io::BufReader;

  let cert_file = &mut BufReader::new(std::fs::File::open(cert_path)?);
  let key_file = &mut BufReader::new(std::fs::File::open(key_path)?);

  let cert_chain = CertificateDer::pem_reader_iter(cert_file).collect::<
    Result<Vec<CertificateDer>, _>
  >()?;

  let keys = PrivatePkcs8KeyDer::pem_reader_iter(key_file)
    .map(|key| key.map(PrivateKeyDer::from))
    .collect::<Result<Vec<_>, _>>()?;

  // Use the first key from the file
  let key = keys
    .into_iter()
    .next()
    .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no private key found"))?;

  let config = rustls::ServerConfig
    ::builder()
    .with_no_client_auth()
    .with_single_cert(cert_chain, key)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

  Ok(config)
}

/// Construct Actix `HttpServer` with conditional middleware based on config flags.
///
/// WHY: Keeps `main` concise; middleware toggles (path normalize, compress, logger) are applied
/// only when enabled to avoid unnecessary overhead.
/// 中文：依設定按需加掛 middleware，減少未使用功能的開銷。
fn build_server(s: &Cofg) -> AppResult<Server> {
  let middleware_cofg = s.middleware.clone();
  let addrs = &s.addrs;

  info!("run in {}://{}/", if s.tls.enable { "https" } else { "http" }, addrs);

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
            .log_target("http-log")
        )
      )
      .wrap(
        middleware::Condition::new(middleware_cofg.http_base_authentication.enable, {
          use std::sync::Arc;
          let users_arc = Arc::new(middleware_cofg.http_base_authentication.users.clone());
          actix_web_httpauth::middleware::HttpAuthentication::basic({
            let users_arc = users_arc.clone();
            move |
              req: actix_web::dev::ServiceRequest,
              credentials: actix_web_httpauth::extractors::basic::BasicAuth
            | {
              let users_arc = users_arc.clone();
              async move {
                let name = credentials.user_id();
                let password = credentials.password();
                let path = req.uri().path();

                if let Some(users) = users_arc.as_ref() {
                  if let Some(user) = users.iter().find(|u| u.name == name) {
                    debug!("http_base_authentication: username match");
                    if ct_eq_str_opt(user.passwords.as_deref(), password) {
                      info!("http_base_authentication: password correct");

                      // allow: 未設定時預設允許所有路徑
                      let in_allow = match user.allow.as_deref() {
                        Some(allow) => allow.iter().any(|allow_path| path.starts_with(allow_path)),
                        None => true,
                      };
                      info!("http_base_authentication: in_allow={}", in_allow);

                      // disallow: 未設定時預設不封鎖任何路徑
                      let not_in_disallow = match user.disallow.as_deref() {
                        Some(disallow) => disallow.iter().all(|p| !path.starts_with(p)),
                        None => true,
                      };
                      info!("http_base_authentication: not_in_disallow={}", not_in_disallow);

                      if in_allow && not_in_disallow {
                        info!("http_base_authentication: ok");
                        return Ok(req);
                      } else {
                        return Err((
                          actix_web::error::ErrorUnauthorized("Unauthorized: path not allowed"),
                          req,
                        ));
                      }
                    } else {
                      // 密碼不符時立即返回，避免落入「無此使用者」訊息
                      return Err((
                        actix_web::error::ErrorUnauthorized(
                          "Unauthorized: no such user name or passwords"
                        ),
                        req,
                      ));
                    }
                  } else {
                    // 沒有任何使用者名稱匹配 → 早退
                    return Err((
                      actix_web::error::ErrorUnauthorized("Unauthorized: no such user name or passwords"),
                      req,
                    ));
                  }
                } else {
                  // NOTE: 未配置任何使用者時一律拒絕，避免無意間全開。
                  warn!("no user data configured");
                }
                Err((actix_web::error::ErrorUnauthorized("Unauthorized: access denied"), req))
              }
            }
          })
        })
      )
      .service(index)
      .service(main_req)
  }).keep_alive(KeepAlive::Os);

  let server = if s.tls.enable {
    let tls_config = load_tls_config(&s.tls.cert, &s.tls.key);
    if let Ok(tls_config) = tls_config {
      server.bind_rustls_0_23(addrs, tls_config)?
    } else {
      warn!("{}, back to no-tls", tls_config.err().unwrap());
      server.bind(addrs)?
    }
  } else {
    server.bind(addrs)?
  };

  Ok(server.run())
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
