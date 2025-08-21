use log::{ debug, info, trace, warn };
use notify::Watcher;
use std::path::PathBuf;
use markdown_ppp::parser::{ parse_markdown, config };
use markdown_ppp::html_printer::{ render_html };
use std::fs::{ copy, create_dir_all, read_to_string, remove_dir_all, remove_file, write };
use wax::Glob;
use actix_web::{ http::KeepAlive, middleware, App, HttpServer };

fn init() -> Result<config::MarkdownParserConfig, Box<dyn std::error::Error>> {
  env_logger
    ::builder()
    .default_format()
    .format_module_path(true)
    .format_line_number(true)
    .format_timestamp(None)
    .filter_level(log::LevelFilter::Info)
    .parse_default_env()
    .init();

  create_dir_all(".\\public\\")?;
  create_dir_all(".\\_public\\")?;
  remove_public()?;
  Ok(config::MarkdownParserConfig::default())
}
fn remove_public() -> Result<(), Box<dyn std::error::Error>> {
  for entry in Glob::new("{*,**/*}")?.walk(".\\public") {
    let entry = entry?;
    let path = entry.path().to_path_buf();
    let ext = path.extension();
    if let Some(e) = ext {
      if e != "md" {
        trace!("init:r={}", path.display());
        if path.is_dir() {
          remove_dir_all(path)?;
        } else if path.is_file() {
          remove_file(path)?;
        }
      }
    } else {
      continue;
    }
  }
  Ok(())
}
fn md2html(c: &config::MarkdownParserConfig) -> Result<(), Box<dyn std::error::Error>> {
  let md_files = Glob::new("**/*.md")?;
  let html_t = read_to_string("html-t.html")?;
  let mut index_file: Option<PathBuf> = None;
  let mut path_lists: Vec<PathBuf> = vec![];

  for entry in md_files.walk("./public/") {
    let entry = entry?;
    let path = entry.path().to_path_buf();
    let out_path_obj = path.with_extension("html");
    let out_path = out_path_obj.to_str().unwrap();

    let input = read_to_string(&path)?;

    if input.starts_with("<!-- TOC -->") {
      index_file = Some(path);
      continue;
    } else {
      path_lists.insert(0, path.clone());
    }
    let ast = parser_md(input, c.clone());
    trace!("ast={ast:#?}");
    write(
      out_path,
      html_t.replace(
        "{}",
        &render_html(&ast, markdown_ppp::html_printer::config::Config::default())
      )
    )?;
  }
  if let Some(index_file) = index_file {
    make_toc(index_file.clone(), path_lists)?;
    let input = read_to_string(index_file.clone())?;
    let ast = parser_md(input, c.clone());
    write(
      index_file.with_extension("html"),
      html_t.replace(
        "{}",
        &render_html(&ast, markdown_ppp::html_printer::config::Config::default())
      )
    )?;
  }
  Ok(())
}
fn make_toc(
  index_file: PathBuf,
  path_list: Vec<PathBuf>
) -> std::result::Result<(), std::io::Error> {
  let mut toc_str_list = vec!["<!-- TOC -->".to_string(), "# toc\n".to_string()];
  for path in path_list {
    let path = path.strip_prefix("./public/").unwrap();
    let path_str = path.with_extension("html").display().to_string();
    let path_str_no_ext = path.with_extension("").display().to_string();

    toc_str_list.insert(toc_str_list.len(), format!("- [{path_str_no_ext}]({path_str})"));
  }
  write(index_file, toc_str_list.join("\n") + "\n")
}
#[inline]
fn parser_md(input: String, c: config::MarkdownParserConfig) -> markdown_ppp::ast::Document {
  parse_markdown(markdown_ppp::parser::MarkdownParserState::with_config(c), &input).unwrap()
}
fn copy_to_public() -> Result<(), Box<dyn std::error::Error>> {
  for entry in Glob::new("{*,**/*}")?.walk(".\\_public") {
    let entry = entry?;
    let path = entry.path().to_path_buf();
    let new_path = PathBuf::from(".\\public").join(path.strip_prefix("./_public")?);

    if let Some(t) = path.to_str() && t.contains(".git") {
      continue;
    }
    debug!("copy_to_public: {} -> {}", path.display(), new_path.display());
    if !path.exists() {
      panic!("{}: not exists", path.display());
    }
    if new_path.exists() && new_path.is_file() {
      debug!("exists:{}", new_path.display());
      continue;
    }
    if path.is_file() {
      trace!("{}: is file", new_path.display());
      copy(&path, &new_path)?;
    } else if path.is_dir() {
      trace!("{}: is dir && not exists", new_path.display());
      create_dir_all(new_path)?;
    } else {
      warn!("{}: ?", new_path.display());
    }
  }
  Ok(())
}

async fn run_server() -> std::io::Result<()> {
  info!("run in http://{}:{}/", IP_PORT.0, IP_PORT.1);
  HttpServer::new(|| {
    App::new()
      .wrap(middleware::NormalizePath::new(middleware::TrailingSlash::Trim))
      .wrap(middleware::Compress::default())
      .wrap(
        middleware::Logger
          ::new(
            &std::env
              ::var("REQUEST_LOGGER")
              .unwrap_or(r#"%{url}xi %s "%{Referer}i" "%{User-Agent}i""#.to_string())
          )
          .custom_request_replace("url", |req| {
            let u = &req.uri().to_string();
            percent_encoding
              ::percent_decode(u.as_bytes())
              .decode_utf8()
              .unwrap_or(std::borrow::Cow::Borrowed(u))
              .to_string()
          })
      )
      .service(
        actix_files::Files
          ::new("/", "./public/")
          .show_files_listing()
          .index_file("index.html")
          .default_handler(
            actix_web::dev::fn_service(|req: actix_web::dev::ServiceRequest| async {
              let (req, _) = req.into_parts();
              let file = actix_files::NamedFile::open_async("./public/404.html").await?;
              let res = file.into_response(&req);
              Ok(actix_web::dev::ServiceResponse::new(req, res))
            })
          )
      )
  })
    .keep_alive(KeepAlive::Os)
    .bind(IP_PORT)?
    .run().await
}

/// ("ip", port)
const IP_PORT: (&str, u16) = ("127.0.0.1", 8080);

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let c = init()?;
  copy_to_public()?;
  md2html(&c)?;

  // spawn the http server in a background thread so the watcher loop can run in main
  let _server_thread = std::thread::spawn(|| {
    actix_web::rt::System::new().block_on(async {
      if let Err(e) = run_server().await {
        eprintln!("server error: {:?}", e);
      }
    });
  });

  let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
  let mut w = notify::recommended_watcher(tx)?;
  w.watch(std::path::Path::new(".\\public\\"), notify::RecursiveMode::Recursive)?;
  w.watch(std::path::Path::new(".\\_public\\"), notify::RecursiveMode::Recursive)?;

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
              .map(|p| p.to_string_lossy().contains(".git\\"))
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
                  Err(e) => {
                    warn!("watch error during drain: {:?}", e);
                  }
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
          let mut need_copy_public = false;
          for ev in events {
            if let Some(p) = ev.paths.first() {
              let pstr = p.to_string_lossy().to_string();
              if pstr.contains(".git\\") {
                continue;
              }
              if pstr.contains("_public\\") {
                need_copy_public = true;
              }
              paths_seen.insert(pstr);
            }
          }

          debug!("processing {} unique paths", paths_seen.len());

          if need_copy_public {
            debug!("ev: is _public");
            remove_public()?;
            copy_to_public()?;
          }

          // regenerate HTML once
          md2html(&c)?;
        }
      }
      Err(e) => println!("watch error: {:?}", e),
    }
  }
  // server runs in background thread; no synchronous run_server() call here

  Ok(())
}
