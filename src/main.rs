#[cfg(windows)]
use std::os::windows::fs::{ symlink_dir, symlink_file };
#[cfg(not(windows))]
use std::os::unix::fs::symlink;
#[cfg(not(windows))]
const symlink_dir: dyn Fn(
  dyn AsRef<std::path::Path>,
  dyn AsRef<std::path::Path>
) -> std::io::Result<()> = symlink;
#[cfg(not(windows))]
const symlink_file: dyn Fn(
  dyn AsRef<std::path::Path>,
  dyn AsRef<std::path::Path>
) -> std::io::Result<()> = symlink;

use std::path::PathBuf;
use std::{ rc::Rc, cell::RefCell };
use markdown_ppp::parser::{ parse_markdown, config };
use markdown_ppp::html_printer::{ render_html };
use markdown_ppp::ast::{ Block, Inline };
use std::fs::{ create_dir_all, read_to_string, remove_dir, remove_file, write };
use wax::Glob;
use actix_web::{ http::KeepAlive, middleware, App, HttpServer };

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy)]
enum GhBlockquoteType {
  NOTE,
  TIP,
  IMPORTANT,
  WARNING,
  CAUTION,
}

impl std::fmt::Display for GhBlockquoteType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      GhBlockquoteType::NOTE => "note",
      GhBlockquoteType::TIP => "tip",
      GhBlockquoteType::IMPORTANT => "important",
      GhBlockquoteType::WARNING => "warning",
      GhBlockquoteType::CAUTION => "caution",
    })
  }
}

fn init() -> Result<config::MarkdownParserConfig, Box<dyn std::error::Error>> {
  create_dir_all("./public/")?;
  for entry in Glob::new("*").unwrap().walk("./public") {
    let entry = entry.unwrap();
    let path = entry.path().to_path_buf();
    if path.is_symlink() {
      if path.is_file() {
        remove_file(path).unwrap();
      } else if path.is_dir() {
        remove_dir(path).unwrap();
      } else {
        match path.extension() {
          None => {
            remove_dir(path).unwrap();
          }
          Some(_) => {
            remove_file(path).unwrap();
          }
        }
      }
    }
  }
  let c = config::MarkdownParserConfig::default().with_block_blockquote_behavior(
    config::ElementBehavior::Map(
      Rc::new(
        RefCell::new(
          Box::new(|input| {
            // println!("i=[{input:#?}]");
            let mut mt: Option<(GhBlockquoteType, &[markdown_ppp::ast::Inline])> = None;

            if let Block::BlockQuote(ref blocks) = input {
              // println!("m:1=t");
              if let Block::Paragraph(inlines) = blocks.first().unwrap() {
                // println!("m:2=t");
                type Gbt = GhBlockquoteType;
                for t in [Gbt::CAUTION, Gbt::IMPORTANT, Gbt::NOTE, Gbt::TIP, Gbt::WARNING] {
                  let m = vec![Inline::Text("!".to_owned() + &t.to_string().to_uppercase())];
                  let mo = &[
                    Inline::LinkReference(markdown_ppp::ast::LinkReference {
                      label: m.clone(),
                      text: m,
                    }),
                  ];
                  let m2 = vec![Inline::Text("!".to_owned() + &t.to_string().to_lowercase())];
                  let m2o = &[
                    Inline::LinkReference(markdown_ppp::ast::LinkReference {
                      label: m2.clone(),
                      text: m2,
                    }),
                  ];
                  // println!("m:3:l={t}[m={m:#?}]");
                  if inlines.starts_with(mo) || inlines.starts_with(m2o) {
                    // println!("m:3=t({t})");
                    if *inlines.get(1).unwrap() == Inline::LineBreak {
                      mt = Some((t, inlines.split_at(2).1));
                    } else {
                      mt = Some((t, inlines.split_at(1).1));
                    }
                  }
                }
              }
            }
            if let Some(mt) = mt {
              let d = mt.1.to_vec();
              // println!("match[{}]", mt.0);
              let mut t1 = vec![];
              match mt.0 {
                GhBlockquoteType::NOTE => {
                  t1.insert(
                    0,
                    Inline::Html(
                      String::from(
                        r#"<div class="markdown-alert markdown-alert-note" dir="auto"><p class="markdown-alert-title" dir="auto"><svg class="octicon octicon-info mr-2" viewBox="0 0 16 16" version="1.1" width="16" height="16" aria-hidden="true"><path d="M0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8Zm8-6.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM6.5 7.75A.75.75 0 0 1 7.25 7h1a.75.75 0 0 1 .75.75v2.75h.25a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1 0-1.5h.25v-2h-.25a.75.75 0 0 1-.75-.75ZM8 6a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"></path></svg>Note</p><p dir="auto">"#
                      )
                    )
                  );
                }
                GhBlockquoteType::TIP => {
                  t1.insert(
                    0,
                    Inline::Html(
                      String::from(
                        r#"<div class="markdown-alert markdown-alert-tip" dir="auto"><p class="markdown-alert-title" dir="auto"><svg class="octicon octicon-light-bulb mr-2" viewBox="0 0 16 16" version="1.1" width="16" height="16" aria-hidden="true"><path d="M8 1.5c-2.363 0-4 1.69-4 3.75 0 .984.424 1.625.984 2.304l.214.253c.223.264.47.556.673.848.284.411.537.896.621 1.49a.75.75 0 0 1-1.484.211c-.04-.282-.163-.547-.37-.847a8.456 8.456 0 0 0-.542-.68c-.084-.1-.173-.205-.268-.32C3.201 7.75 2.5 6.766 2.5 5.25 2.5 2.31 4.863 0 8 0s5.5 2.31 5.5 5.25c0 1.516-.701 2.5-1.328 3.259-.095.115-.184.22-.268.319-.207.245-.383.453-.541.681-.208.3-.33.565-.37.847a.751.751 0 0 1-1.485-.212c.084-.593.337-1.078.621-1.489.203-.292.45-.584.673-.848.075-.088.147-.173.213-.253.561-.679.985-1.32.985-2.304 0-2.06-1.637-3.75-4-3.75ZM5.75 12h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1 0-1.5ZM6 15.25a.75.75 0 0 1 .75-.75h2.5a.75.75 0 0 1 0 1.5h-2.5a.75.75 0 0 1-.75-.75Z"></path></svg>Tip</p><p dir="auto">"#
                      )
                    )
                  );
                }
                GhBlockquoteType::IMPORTANT => {
                  t1.insert(
                    0,
                    Inline::Html(
                      String::from(
                        r#"<div class="markdown-alert markdown-alert-important" dir="auto"><p class="markdown-alert-title" dir="auto"><svg class="octicon octicon-report mr-2" viewBox="0 0 16 16" version="1.1" width="16" height="16" aria-hidden="true"><path d="M0 1.75C0 .784.784 0 1.75 0h12.5C15.216 0 16 .784 16 1.75v9.5A1.75 1.75 0 0 1 14.25 13H8.06l-2.573 2.573A1.458 1.458 0 0 1 3 14.543V13H1.75A1.75 1.75 0 0 1 0 11.25Zm1.75-.25a.25.25 0 0 0-.25.25v9.5c0 .138.112.25.25.25h2a.75.75 0 0 1 .75.75v2.19l2.72-2.72a.749.749 0 0 1 .53-.22h6.5a.25.25 0 0 0 .25-.25v-9.5a.25.25 0 0 0-.25-.25Zm7 2.25v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 9a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"></path></svg>Important</p><p dir="auto">"#
                      )
                    )
                  );
                }
                GhBlockquoteType::WARNING => {
                  t1.insert(
                    0,
                    Inline::Html(
                      String::from(
                        r#"<div class="markdown-alert markdown-alert-warning" dir="auto"><p class="markdown-alert-title" dir="auto"><svg class="octicon octicon-alert mr-2" viewBox="0 0 16 16" version="1.1" width="16" height="16" aria-hidden="true"><path d="M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575Zm1.763.707a.25.25 0 0 0-.44 0L1.698 13.132a.25.25 0 0 0 .22.368h12.164a.25.25 0 0 0 .22-.368Zm.53 3.996v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 11a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"></path></svg>Warning</p><p dir="auto">"#
                      )
                    )
                  );
                }
                GhBlockquoteType::CAUTION => {
                  t1.insert(
                    0,
                    Inline::Html(
                      String::from(
                        r#"<div class="markdown-alert markdown-alert-caution" dir="auto"><p class="markdown-alert-title" dir="auto"><svg class="octicon octicon-stop mr-2" viewBox="0 0 16 16" version="1.1" width="16" height="16" aria-hidden="true"><path d="M4.47.22A.749.749 0 0 1 5 0h6c.199 0 .389.079.53.22l4.25 4.25c.141.14.22.331.22.53v6a.749.749 0 0 1-.22.53l-4.25 4.25A.749.749 0 0 1 11 16H5a.749.749 0 0 1-.53-.22L.22 11.53A.749.749 0 0 1 0 11V5c0-.199.079-.389.22-.53Zm.84 1.28L1.5 5.31v5.38l3.81 3.81h5.38l3.81-3.81V5.31L10.69 1.5ZM8 4a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 8 4Zm0 8a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"></path></svg>Caution</p><p dir="auto">"#
                      )
                    )
                  );
                }
              }

              t1.extend(d);
              t1.insert(t1.len(), Inline::Html(String::from("</p></div>")));

              Block::Paragraph(t1)
            } else {
              // println!("nm:{input:#?}");
              input
            }
          })
        )
      )
    )
  );
  Ok(c)
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
    // println!("i={input}");
    if input.starts_with("<!-- TOC -->") {
      index_file = Some(path);
      continue;
    } else {
      path_lists.insert(0, path.clone());
    }
    let ast = parser_md(input, c.clone());
    // println!("ast={ast:#?}");
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
  for entry in Glob::new("*").unwrap().walk("./_public") {
    let entry = entry.unwrap();
    let path = entry.path().to_path_buf();
    let new_path = PathBuf::from(".\\public").join(path.strip_prefix("./_public").unwrap());
    if new_path.exists() {
      println!("exists:{}", new_path.display());
      continue;
    }
    if path.is_file() {
      symlink_file(path, new_path).unwrap();
    } else if path.is_dir() {
      symlink_dir(path, new_path).unwrap();
    }
  }
}

const IP_PORT: (&str, u16) = ("127.0.0.1", 8080);
#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let c = init().unwrap();
  copy_to_public();
  md2html(c).unwrap();

  println!("run in http://{}:{}/", IP_PORT.0, IP_PORT.1);
  HttpServer::new(|| {
    App::new()
      .wrap(middleware::Logger::default())
      .wrap(middleware::Compress::default())
      .wrap(middleware::NormalizePath::default())
      .service(
        actix_files::Files::new("/", "./public/").show_files_listing().index_file("index.html")
      )
  })
    .keep_alive(KeepAlive::Os)
    .bind(IP_PORT)?
    .run().await
}
