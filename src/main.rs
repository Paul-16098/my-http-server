use log::{ debug, info, trace, warn };
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
    .filter_level(log::LevelFilter::Info)
    .parse_default_env()
    .init();

  create_dir_all(".\\public\\")?;
  for entry in Glob::new("{*,**/*}").unwrap().walk(".\\public") {
    let entry = entry.unwrap();
    let path = entry.path().to_path_buf();
    let ext = path.extension();
    if ext.is_none() {
      continue;
    }
    if ext.unwrap() != "md" {
      trace!("init:r={}", path.display());
      if path.is_dir() {
        remove_dir_all(path).unwrap();
      } else if path.is_file() {
        remove_file(path).unwrap();
      }
    }
  }
  Ok(config::MarkdownParserConfig::default())
}

fn md2html(c: config::MarkdownParserConfig) -> Result<(), Box<dyn std::error::Error>> {
  let md_files = Glob::new("**/*.md").unwrap();
  let html_t = read_to_string("html-t.html")?;
  let mut index_file: Option<PathBuf> = None;
  let mut path_lists: Vec<PathBuf> = vec![];

  for entry in md_files.walk("./public/") {
    let entry = entry.unwrap();
    let path = entry.path().to_path_buf();
    let opatho = path.with_extension("html");
    let opath = opatho.to_str().unwrap();

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
      opath,
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
fn parser_md(input: String, c: config::MarkdownParserConfig) -> markdown_ppp::ast::Document {
  parse_markdown(markdown_ppp::parser::MarkdownParserState::with_config(c), &input).unwrap()
}
fn copy_to_public() {
  for entry in Glob::new("{*,**/*}").unwrap().walk(".\\_public") {
    let entry = entry.unwrap();
    let path = entry.path().to_path_buf();
    let new_path = PathBuf::from(".\\public").join(path.strip_prefix("./_public").unwrap());
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
      copy(&path, &new_path).unwrap();
    } else if path.is_dir() {
      trace!("{}: is dir && not exists", new_path.display());
      create_dir_all(new_path).unwrap();
    } else {
      warn!("{}: ?", new_path.display());
    }
  }
}

const IP_PORT: (&str, u16) = ("127.0.0.1", 8080);
#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let c = init().unwrap();
  copy_to_public();
  md2html(c).unwrap();

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
        actix_files::Files::new("/", "./public/").show_files_listing().index_file("index.html")
      )
  })
    .keep_alive(KeepAlive::Os)
    .bind(IP_PORT)?
    .run().await
}
