//! main

#[cfg(test)]
mod test;

mod api;
mod cofg;
mod parser;
use crate::cofg::{cli, config::Cofg};
mod error;
use crate::error::AppResult;
mod request;
use crate::request::main_req;

use actix_web::{App, HttpServer, dev::Server, http::KeepAlive, middleware};
use clap::Parser as _;
use log::{debug, error, info, warn};
use std::fs::create_dir_all;
use std::path::Path;

/// Initialize logging & ensure `public_path` directory exists.
///
/// WHY: Keep side-effect setup isolated from `main()`. Directory creation early prevents
/// per-request race to create it lazily. Logger configured with module paths for traceability.
fn init(c: &Cofg) -> AppResult<()> {
    logger_init();

    create_dir_all(&c.public_path)?;
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

// SECURITY: Constant-time comparison to reduce timing attack surface.
#[cfg(test)]
pub(crate) fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());
    let mut diff: u8 = (a.len() ^ b.len()) as u8;
    for i in 0..max_len {
        let ai = *a.get(i).unwrap_or(&0);
        let bi = *b.get(i).unwrap_or(&0);
        diff |= ai ^ bi;
    }
    diff == 0
}

#[cfg(not(test))]
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

// Constant-time comparison for Option<&str>
#[cfg(test)]
pub(crate) fn ct_eq_str_opt(a: Option<&str>, b: Option<&str>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => constant_time_eq(a.as_bytes(), b.as_bytes()),
        (None, None) => true,
        _ => false,
    }
}

#[cfg(not(test))]
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
fn load_tls_config(cert_path: &str, key_path: &str) -> AppResult<rustls::ServerConfig> {
    use rustls::pki_types::{
        CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, pem::PemObject as _,
    };
    use std::io::BufReader;

    let cert_file = &mut BufReader::new(std::fs::File::open(cert_path)?);
    let key_file = &mut BufReader::new(std::fs::File::open(key_path)?);

    let cert_chain =
        CertificateDer::pem_reader_iter(cert_file).collect::<Result<Vec<CertificateDer>, _>>()?;

    let keys = PrivatePkcs8KeyDer::pem_reader_iter(key_file)
        .map(|key| key.map(PrivateKeyDer::from))
        .collect::<Result<Vec<_>, _>>()?;

    // Use the first key from the file
    let key = keys.into_iter().next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "no private key found")
    })?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    Ok(config)
}

/// Construct Actix `HttpServer` with conditional middleware based on config flags.
///
/// WHY: Keeps `main` concise; middleware toggles (path normalize, compress, logger) are applied
/// only when enabled to avoid unnecessary overhead.
fn build_server(s: &Cofg) -> AppResult<Server> {
    let middleware_cofg = s.middleware.clone();
    let addrs = &s.addrs;

    info!(
        "run in {}://{}/",
        if s.tls.enable { "https" } else { "http" },
        addrs
    );

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Condition::new(
                middleware_cofg.rate_limiting.enable,
                {
                    let cfg = actix_governor::GovernorConfigBuilder::default()
                        .seconds_per_request(middleware_cofg.rate_limiting.seconds_per_request)
                        .burst_size(middleware_cofg.rate_limiting.burst_size)
                        .finish()
                        .unwrap();
                    actix_governor::Governor::new(&cfg)
                },
            ))
            .wrap(middleware::Condition::new(
                middleware_cofg.logger.enabling,
                middleware::Logger::new(&middleware_cofg.logger.format)
                    .custom_request_replace("url", |req| {
                        let u = &req.uri().to_string();
                        let mut u = percent_encoding::percent_decode(u.as_bytes())
                            .decode_utf8()
                            .unwrap_or(std::borrow::Cow::Borrowed(u))
                            .to_string();
                        if u.starts_with('/') {
                            u.remove(0);
                        }
                        u
                    })
                    .log_target("http-log"),
            ))
            .wrap(middleware::Condition::new(
                middleware_cofg.normalize_path,
                middleware::NormalizePath::new(middleware::TrailingSlash::Trim),
            ))
            .wrap(middleware::Condition::new(
                middleware_cofg.compress,
                middleware::Compress::default(),
            ))
            .wrap(middleware::Condition::new(
                middleware_cofg.http_base_authentication.enable,
                {
                    use std::sync::Arc;
                    let users_arc =
                        Arc::new(middleware_cofg.http_base_authentication.users.clone());
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

                      let in_allow = match user.allow.as_deref() {
                        Some(allow) => allow.iter().any(|allow_path| path.starts_with(allow_path)),
                        None => true,
                      };
                      info!("http_base_authentication: in_allow={}", in_allow);

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
                      return Err((
                        actix_web::error::ErrorUnauthorized(
                          "Unauthorized: no such user name or passwords"
                        ),
                        req,
                      ));
                    }
                  } else {
                    return Err((
                      actix_web::error::ErrorUnauthorized(
                        "Unauthorized: no such user name or passwords"
                      ),
                      req,
                    ));
                  }
                } else {
                  warn!("no user data configured");
                }
                Err((actix_web::error::ErrorUnauthorized("Unauthorized: access denied"), req))
              }
            }
                    })
                },
            ))
            .wrap(middleware::Condition::new(
                middleware_cofg.ip_filter.enable,
                {
                    use actix_ip_filter::IPFilter;
                    let mut filter = IPFilter::new();

                    // If allow list is specified, use whitelist mode
                    if let Some(allow_list) = middleware_cofg.ip_filter.allow.as_ref() {
                        let allow_refs: Vec<&str> = allow_list.iter().map(|s| s.as_str()).collect();
                        filter = filter.allow(allow_refs);
                    }

                    // If block list is specified, add to blocklist
                    if let Some(block_list) = middleware_cofg.ip_filter.block.as_ref() {
                        let block_refs: Vec<&str> = block_list.iter().map(|s| s.as_str()).collect();
                        filter = filter.block(block_refs);
                    }

                    filter
                },
            ))
            .service(main_req)
    })
    .keep_alive(KeepAlive::Os);

    let server = if s.tls.enable {
        match load_tls_config(&s.tls.cert, &s.tls.key) {
            Ok(tls_config) => server.bind_rustls_0_23(addrs, tls_config)?,
            Err(e) => {
                warn!("{}, back to no-tls", e);
                server.bind(addrs)?
            }
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
            warn!(
                "Failed to canonicalize public_path '{}': {}",
                &s.public_path, e
            );
            (&s.public_path).into()
        })
        .to_string_lossy()
        .to_string();

    init(&s)?;
    info!(
        "VERSION: {} in {}",
        option_env!("VERSION").unwrap_or("?"),
        Path::new(".").canonicalize()?.display()
    );
    debug!("cofg: {s:#?}");

    build_server(&s)?.await?;
    Ok(())
}
