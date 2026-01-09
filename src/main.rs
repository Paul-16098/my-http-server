//! main

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod test;

#[cfg(feature = "api")]
mod api;
mod cofg;
mod parser;
use crate::cofg::{cli, config::Cofg};
mod error;
use crate::error::AppResult;
mod request;
use crate::request::main_req;

use actix_web::HttpResponse;
use actix_web::{App, HttpServer, dev::Server, http::KeepAlive, middleware};
use clap::Parser as _;
use log::{debug, error, info, warn};
use std::fs::create_dir_all;
use std::path::Path;

#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[derive(serde::Serialize)]
pub(crate) struct Version {
    version: &'static str,
    profile: &'static str,
    commit_hash: &'static str,
    env_suffix: &'static str,
    features: &'static str,
}
const VERSION: Version = Version::new();

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({} Profile)-{}({})[f:{}]",
            self.version,
            self.profile,
            self.commit_hash,
            self.env_suffix,
            if self.features.is_empty() {
                "none"
            } else {
                self.features
            },
        )
    }
}

impl Version {
    pub(crate) const fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            profile: env!("PROFILE"),
            commit_hash: env!("commit_hash"),
            env_suffix: env!("env_suffix"),
            features: env!("FEATURES"),
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize logging & ensure `public_path` directory exists.
///
/// WHY: Keep side-effect setup isolated from `main()`. Directory creation early prevents
/// per-request race to create it lazily. Logger configured with module paths for traceability.
fn init(_c: &Cofg) -> AppResult<()> {
    if let Some(xdg_paths) = Cofg::get_xdg_paths() {
        if let Some(parent) = xdg_paths.cofg.parent() {
            create_dir_all(parent)?;
        }

        if !xdg_paths.cofg.exists() {
            std::fs::write(&xdg_paths.cofg, include_str!("./cofg/cofg.yaml"))?;
            info!("Created default XDG config at {}", xdg_paths.cofg.display());
        }

        if !xdg_paths.template_hbs.exists() {
            std::fs::write(&xdg_paths.template_hbs, include_str!("../meta/html-t.hbs"))?;
            info!(
                "Created default XDG template at {}",
                xdg_paths.template_hbs.display()
            );
        }

        if !xdg_paths.page_404.exists() {
            std::fs::write(&xdg_paths.page_404, include_str!("../meta/404.html"))?;
            info!(
                "Created default XDG 404 page at {}",
                xdg_paths.page_404.display()
            );
        }
    }
    #[cfg(feature = "github_emojis")]
    emojis_init(std::env::var("GITHUB_TOKEN").ok())?;
    Ok(())
}
#[cfg(feature = "github_emojis")]
fn emojis_init(ght: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    use parser::{EMOJIS, Emojis};
    use std::collections::HashMap;

    // Determine emoji cache path with XDG fallback
    let emoji_path = if let Some(xdg_paths) = cofg::config::Cofg::get_xdg_paths() {
        // Ensure XDG directory exists
        if let Some(parent) = xdg_paths.emojis.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        xdg_paths.emojis
    } else {
        Path::new("./emojis.json").to_path_buf()
    };

    if !emoji_path.exists() {
        info!("emoji json file not found at {}, fetching from github api...", emoji_path.display());
        let mut resp = ureq::get("https://api.github.com/emojis")
            .header("User-Agent", "Paul-16098/my-http-server");

        if let Some(t) = ght {
            resp = resp.header("Authorization", format!("Bearer {}", t));
        }
        let mut resp = resp.call()?;
        let body = resp.body_mut().read_json::<HashMap<String, String>>()?;
        let mut unicode_emojis = HashMap::new();
        let mut else_emojis = HashMap::new();
        for (k, v) in body.iter() {
            if v.contains("unicode") {
                let unicode = v
                    .trim_end_matches(".png?v8")
                    .split('/')
                    .collect::<Vec<_>>()
                    .last()
                    .map(|last| {
                        last.split('-')
                            .filter_map(|code| u32::from_str_radix(code, 16).ok())
                            .filter_map(std::char::from_u32)
                            .collect::<String>()
                    });
                match unicode {
                    Some(unicode) => {
                        debug!("Found unicode emoji: {k} -> {unicode}");
                        unicode_emojis.insert(k.clone(), unicode);
                    }
                    None => {
                        warn!("Failed to parse unicode emoji for key: {k}, value: {v}");
                    }
                }
            } else {
                debug!("Found non-unicode emoji: {k} -> {v}");
                else_emojis.insert(k.clone(), v.clone());
            }
        }

        let json = serde_json::to_string(&Emojis {
            unicode: unicode_emojis,
            r#else: else_emojis,
        })?;
        std::fs::write(&emoji_path, json)?;
        info!("Saved emoji cache to {}", emoji_path.display());
    } else {
        info!("emoji json file found at {}, skipping fetch.", emoji_path.display());
    }
    let emojis_json = std::fs::read_to_string(&emoji_path).map_err(|e| {
        error!("Failed to read emojis.json from {}: {}", emoji_path.display(), e);
        e
    })?;
    let emojis: parser::Emojis = serde_json::from_str(&emojis_json).map_err(|e| {
        error!("Failed to parse emojis.json as valid JSON: {}", e);
        e
    })?;
    EMOJIS.get_or_init(|| emojis);
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

// Constant-time comparison for Option<&str>
pub(crate) fn ct_eq_str_opt(a: Option<&str>, b: Option<&str>) -> bool {
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
    #[cfg(feature = "api")]
    let api_enable = s.api.enable;

    info!(
        "run in {}://{}/",
        if s.tls.enable { "https" } else { "http" },
        addrs
    );

    let server = HttpServer::new(move || {
        let mut app = App::new()
            .wrap(middleware::Condition::new(
                middleware_cofg.rate_limiting.enable,
                {
                    let cfg = actix_governor::GovernorConfigBuilder::default()
                        .seconds_per_request(middleware_cofg.rate_limiting.seconds_per_request)
                        .burst_size(middleware_cofg.rate_limiting.burst_size)
                        .finish()
                        .unwrap_or_else(|| {
                            error!("Failed to build rate limiting config");
                            actix_governor::GovernorConfig::default()
                        });
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

                    for rule in middleware_cofg.ip_filter.rules.iter() {
                        // If allow list is specified, use whitelist mode
                        if let Some(allow_list) = rule.allow.as_ref() {
                            let allow_refs: Vec<&str> =
                                allow_list.iter().map(|s| s.as_str()).collect();
                            filter = filter.allow(allow_refs);
                        }

                        // If block list is specified, add to blocklist
                        if let Some(block_list) = rule.block.as_ref() {
                            let block_refs: Vec<&str> =
                                block_list.iter().map(|s| s.as_str()).collect();
                            filter = filter.block(block_refs);
                        }
                        filter =
                            filter.limit_to(rule.limit_to.iter().map(|f| f.as_str()).collect());
                    }
                    filter = filter.on_block(
                        |_flt: &IPFilter, ip: &str, req: &actix_web::dev::ServiceRequest| {
                            debug!("ip_filter: block ip {ip} req={:?}", req);
                            Some(
                                HttpResponse::Forbidden()
                                    .body(format!("IP is blocked, your IP is {ip}")),
                            )
                        },
                    );

                    filter
                },
            ));
        #[cfg(feature = "api")]
        if api_enable {
            app = app.service(
                actix_web::web::scope("/api")
                    .service(api::docs)
                    .service(api::raw_openapi)
                    .service(api::meta)
                    .service(api::license)
                    .service(api::file::get_raw_file)
                    .service(api::file::file_info)
                    .service(api::file::list_files)
                    .service(api::file::check_exists),
            );
        }
        app = app.service(main_req);
        app
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
    logger_init();

    // Parse CLI arguments
    let cli_args = cli::Args::parse();

    // Handle root_dir early (change CWD before config load)
    if let Some(ref dir) = cli_args.root_dir {
        std::env::set_current_dir(dir).map_err(|e| {
            error!("Failed to change directory to '{}': {}", dir, e);
            crate::error::AppError::CliError(format!("ROOT_DIR must be a valid path: {}", e))
        })?;
        info!("Changed working directory to: {}", dir);
    }

    // Initialize global config with full layered precedence
    let mut s = Cofg::init_global(&cli_args)?;

    // Canonicalize public_path for consistent path resolution
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
        VERSION,
        Path::new(".").canonicalize()?.display()
    );
    debug!("cofg: {s:#?}");

    build_server(&s)?.await?;
    Ok(())
}
