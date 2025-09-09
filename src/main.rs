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
use std::sync::{ atomic::{ AtomicBool, Ordering }, Arc };
use wax::Glob;
use actix_web::{ dev::{ Server, ServerHandle }, http::KeepAlive, middleware, App, HttpServer };

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
    .run();

  Ok(server)
}

const DEBOUNCE_DURATION_MS: u64 = 1000;
const DEBOUNCE_CHECK_INTERVAL_MS: u64 = 100;

fn watcher_loop(stop: Arc<AtomicBool>) -> AppResult<()> {
  let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
  let mut w = notify::recommended_watcher(tx)?;
  let public_path = cofg::Cofg::get(false).public_path;
  w.watch(std::path::Path::new(&public_path), notify::RecursiveMode::Recursive)?;

  // debounce loop: collect events for a short interval and process unique paths once
  loop {
    if stop.load(Ordering::Relaxed) {
      debug!("watcher_loop stop requested");
      break;
    }
    // use timeout to periodically check stop flag
    match rx.recv_timeout(std::time::Duration::from_millis(250)) {
      Ok(Ok(first_event)) => {
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
          // sleep but still allow early stop checks
          let mut slept = 0u64;
          while slept < DEBOUNCE_DURATION_MS {
            if stop.load(Ordering::Relaxed) {
              break;
            }
            std::thread::sleep(std::time::Duration::from_millis(DEBOUNCE_CHECK_INTERVAL_MS));
            slept += DEBOUNCE_CHECK_INTERVAL_MS;
          }

          // drain any pending events into the buffer
          loop {
            if stop.load(Ordering::Relaxed) {
              break;
            }
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
          if !stop.load(Ordering::Relaxed) {
            let cfg = cofg::Cofg::get(true);
            md2html_all()?;
            if cfg.toc.make_toc {
              make_toc()?;
            }
          }
        }
      }
      Ok(Err(e)) => {
        warn!("watch error: {e:?}");
      }
      Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
        // periodic check, just continue
        continue;
      }
      Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
        info!("watch channel disconnected, exiting watcher_loop");
        break;
      }
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

  // graceful shutdown flag
  let stop = Arc::new(AtomicBool::new(false));

  let s_c = s.clone();
  // channel to receive ServerHandle and a done signal when the server exits
  let (handle_tx, handle_rx) = std::sync::mpsc::channel::<ServerHandle>();
  let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
  // spawn the http server in a background thread so the watcher loop can run in main
  let server_thread = std::thread::spawn(move || {
    match build_server(&s_c) {
      Ok(server) => {
        let handle = server.handle();
        let _ = handle_tx.send(handle);
        actix_web::rt::System::new().block_on(async move {
          if let Err(e) = server.await {
            error!("server error: {e:?}");
          }
          // notify main that server has fully exited
          let _ = done_tx.send(());
        });
      }
      Err(e) => {
        error!("failed to build server: {e:?}");
      }
    }
  });
  // wait for server handle to be ready
  let server_handle = handle_rx
    .recv()
    .expect("server thread failed to send handle or thread panicked during server initialization");

  // set Ctrl+C handler once we have the server handle
  let stop_for_sig = stop.clone();
  ctrlc
    ::set_handler(move || {
      info!("SIGINT received; shutting down...");
      // only set stop flag; Actix handles its own shutdown on SIGINT
      stop_for_sig.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

  // also stop watcher when server thread finishes (in case SIGINT wasn't caught here)
  {
    let stop_on_done = stop.clone();
    std::thread::spawn(move || {
      let _ = done_rx.recv();
      stop_on_done.store(true, Ordering::SeqCst);
    });
  }
  if s.watch {
    // start watcher loop (will exit on Ctrl+C)
    watcher_loop(stop.clone())?;
  } else {
    // block until Ctrl+C
    while !stop.load(Ordering::SeqCst) {
      std::thread::sleep(std::time::Duration::from_millis(200));
    }
  }

  // ensure server is stopping (idempotent hint to Actix)
  std::mem::drop(server_handle.stop(true));
  if let Err(e) = server_thread.join() {
    error!("server thread join error: {:?}", e);
  }

  Ok(())
}
