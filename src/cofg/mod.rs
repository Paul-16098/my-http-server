//! cofg main

use nest_struct::nest_struct;
use once_cell::sync::OnceCell;
use std::sync::{ RwLock };

pub(crate) const BUILD_COFG: &str = include_str!("cofg.yaml");

#[nest_struct]
#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct Cofg {
  pub(crate) addrs: nest! {
    /// like: 127.0.0.1
    pub(crate) ip: String,
    /// like: 80, 8080
    pub(crate) port: u16,
  },
  pub(crate) middleware: nest! {
    /// enabling NormalizePath
    pub(crate) normalize_path: bool,
    /// enabling Compress
    pub(crate) compress: bool,
    pub(crate) logger: nest! {
      /// enabling logger
      pub(crate) enabling: bool,
      /// logger format
      pub(crate) format: String
    }
  },
  /// watch file changes
  // pub(crate) watch: bool,
  pub(crate) templating: nest! {
    pub(crate) value: Option<Vec<String>>,
    pub(crate) hot_reload: bool
  },
  pub(crate) toc: nest! {
    // pub(crate) make_toc: bool,
    pub(crate) path: String,
    pub(crate) ext: Vec<String>
  },
  pub(crate) public_path: String,
}

// global cached config; allow refresh when hot_reload = true
static GLOBAL_COFG: OnceCell<RwLock<Cofg>> = OnceCell::new();

impl Cofg {
  fn load_from_disk() -> Self {
    if !std::path::Path::new("./cofg.yaml").exists() {
      println!("write default cofg");
      std::fs::write("./cofg.yaml", BUILD_COFG).unwrap();
    }
    config::Config
      ::builder()
      .add_source(config::File::with_name("./cofg.yaml").required(false))
      .build()
      .unwrap()
      .try_deserialize::<Cofg>()
      .unwrap()
  }

  /// Get cached config (lazy init). If `force_reload` is true and current config has
  /// `templating.hot_reload` enabled, it'll reload from disk.
  pub(crate) fn new() -> Self {
    Self::get(false)
  }

  pub(crate) fn get(force_reload: bool) -> Self {
    let cell = GLOBAL_COFG.get_or_init(|| RwLock::new(Self::load_from_disk()));
    if
      force_reload &&
      cell
        .read()
        .map(|r| r.templating.hot_reload)
        .unwrap_or(false) &&
      let Ok(mut w) = cell.write()
    {
      *w = Self::load_from_disk();
    }
    cell
      .read()
      .map(|g| g.clone())
      .unwrap_or_default()
  }

  /// Force refresh ignoring hot_reload flag (used rarely / tests)
  #[allow(dead_code)]
  pub(crate) fn force_refresh() {
    if let Some(lock) = GLOBAL_COFG.get() && let Ok(mut w) = lock.write() {
      *w = Self::load_from_disk();
    }
  }
}
impl std::fmt::Display for CofgAddrs {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.ip, self.port)
  }
}
impl std::net::ToSocketAddrs for CofgAddrs {
  type Iter = std::vec::IntoIter<std::net::SocketAddr>;

  fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
    std::net::ToSocketAddrs::to_socket_addrs(&(self.ip.as_str(), self.port))
  }
}

impl Default for Cofg {
  fn default() -> Self {
    config::Config
      ::builder()
      .add_source(config::File::from_str(BUILD_COFG, config::FileFormat::Yaml))
      .build()
      .unwrap()
      .try_deserialize::<Self>()
      .unwrap()
  }
}
