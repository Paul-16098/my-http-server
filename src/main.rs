//! main

#[cfg(test)]
mod test;

mod parser;
use crate::parser::markdown::{ make_toc, md2html_all };
mod cofg;
use crate::cofg::Cofg;
mod error;
use crate::error::{ AppResult, AppError };

use log::{ debug, error, info, warn };
use notify::Watcher;
use std::fs::{ create_dir_all, remove_file };
use wax::Glob;
use actix_web::{ http::KeepAlive, middleware, App, HttpServer };

fn init() -> AppResult<()> {
  env_logger
    ::builder()
    .default_format()
    .format_timestamp(None)
    .filter_level(log::LevelFilter::Info)
    .parse_default_env()
    .init();

  create_dir_all(cofg::Cofg::get(false).public_path)?;
  remove_public()?;
  Ok(())
}
fn remove_public() -> AppResult<()> {
  let public_path = cofg::Cofg::get(false).public_path;
  // Map glob build error into our AppError
  let glob = Glob::new("**/*.{md,markdown}")?;
  for entry in glob.walk(public_path) {
    let entry = entry?;
    let path = entry.path().to_path_buf();

    let out = path.with_extension("html");
    if out.exists() && let Err(e) = remove_file(out) {
      warn!("remove_public remove_file error: {e}");
    }
  }
  Ok(())
}
async fn run_server(s: &Cofg) -> std::io::Result<()> {
  let middleware_cofg = s.middleware.clone();
  let addrs = &s.addrs;
  info!("run in http://{}/", addrs);

  HttpServer::new(move || {
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
              percent_encoding
                ::percent_decode(u.as_bytes())
                .decode_utf8()
                .unwrap_or(std::borrow::Cow::Borrowed(u))
                .to_string()
            })
        )
      )
      .service(
        actix_files::Files
          ::new("/", cofg::Cofg::new().public_path)
          .show_files_listing()
          .index_file("index.html")
          .default_handler(
            actix_web::dev::fn_service(|req: actix_web::dev::ServiceRequest| async {
              let (req, _) = req.into_parts();
              let file = actix_files::NamedFile::open_async("./meta/404.html").await?;
              let res = file.into_response(&req);
              Ok(actix_web::dev::ServiceResponse::new(req, res))
            })
          )
      )
  })
    .keep_alive(KeepAlive::Os)
    .bind(addrs.to_string())?
    .run().await
}

fn watcher_loop() -> AppResult<()> {
  let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
  let mut w = notify::recommended_watcher(tx)?;
  let public_path = cofg::Cofg::get(false).public_path;
  w.watch(std::path::Path::new(&public_path), notify::RecursiveMode::Recursive)?;

  // debounce loop: collect events for a short interval and process unique paths once
  for res in &rx {
    match res {
      Ok(first_event) => {
        if
          matches!(
            first_event.kind,
            notify::EventKind::Modify(_) |
              notify::EventKind::Create(_) |
              notify::EventKind::Remove(_)
          )
        {
          // 忽略 git 路徑
          if
            first_event.paths
              .first()
              .map(|p| p.to_string_lossy().contains(".git"))
              .unwrap_or(false)
          {
            continue;
          }

          // start debounce: collect events for a short time window
          debug!("debounce start: {first_event:#?}");
          let mut events: Vec<notify::Event> = vec![first_event];
          std::thread::sleep(std::time::Duration::from_millis(1000));

          // drain any pending events into the buffer
          loop {
            match rx.try_recv() {
              Ok(next) =>
                match next {
                  Ok(ev) => events.push(ev),
                  Err(e) => warn!("watch error during drain: {e:?}"),
                }
              Err(std::sync::mpsc::TryRecvError::Empty) => {
                break;
              }
              Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                break;
              }
            }
          }

          // deduplicate by path and decide actions
          let mut paths_seen = std::collections::HashSet::new();
          for ev in events {
            if let Some(p) = ev.paths.first() {
              let pstr = p.to_string_lossy().to_string();
              if pstr.contains(".git") {
                continue;
              }
              paths_seen.insert(pstr);
            }
          }

          debug!("processing {} unique paths", paths_seen.len());

          // regenerate HTML once
          // reload config if hot_reload enabled
          let cfg = cofg::Cofg::get(true);
          md2html_all()?;
          if cfg.toc.make_toc {
            make_toc()?;
          }
        }
      }
      Err(e) => println!("watch error: {e:?}"),
    }
  }

  Ok(())
}

fn main() -> Result<(), AppError> {
  let s = cofg::Cofg::get(false);
  init()?;
  debug!("cofg: {s:#?}");
  md2html_all()?;
  if s.toc.make_toc {
    make_toc()?;
  }

  let s_c = s.clone();
  // spawn the http server in a background thread so the watcher loop can run in main
  std::thread::spawn(move || {
    actix_web::rt::System::new().block_on(async {
      if let Err(e) = run_server(&s_c).await {
        error!("server error: {e:?}");
      }
    });
  });
  if s.watch {
    // start watcher loop
    watcher_loop()?;
  }

  Ok(())
}
