//! Configuration (Cofg)
//!
//! WHY: Centralized runtime configuration cached in a global `OnceCell<RwLock<_>>` so hot paths
//! (HTTP request handling & markdown rendering) avoid disk IO / deserialization cost. A reload is
//! only attempted when caller explicitly asks (`get(true)`) AND hot-reload is enabled. This keeps
//! the steady-state fast while still offering a development-friendly live tweaking mode.
//!
//! 中文說明：集中式設定透過 `OnceCell` 快取，避免每次請求重新讀取/解析；只有在呼叫方要求且
//! 設定檔允許 hot_reload 時才重讀，兼顧執行期效能與開發彈性。

use nest_struct::nest_struct;
use once_cell::sync::OnceCell;
use std::sync::{ RwLock };

pub(crate) const BUILD_COFG: &str = include_str!("cofg.yaml");

#[nest_struct]
#[derive(PartialEq, Clone, Debug, serde::Deserialize)]
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
    pub(crate) ext: Vec<String>,
    pub(crate) ig: Vec<String>
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

  /// Returns cached configuration (lazy init). Equivalent to `Cofg::get(false)`.
  ///
  /// WHY: Keep call sites terse in hot paths (e.g. HTTP handlers) while expressing intent of
  /// "give me the (maybe) cached config".
  ///
  /// 中文：回傳快取設定（延遲初始化），語意精簡，避免熱路徑多寫參數。
  pub(crate) fn new() -> Self {
    Self::get(false)
  }

  /// Obtain configuration, optionally forcing a reload when hot_reload is enabled.
  ///
  /// `force_reload = true` triggers a disk re-read ONLY IF `templating.hot_reload` is true.
  /// This prevents accidental perf regressions in production where the file should be static.
  ///
  /// 中文：`force_reload` 僅在 hot_reload 啟用時實際重讀設定，避免正式環境多餘 IO。
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

  /// Force a refresh ignoring the hot_reload flag (primarily for tests & rare admin scenarios).
  ///
  /// WHY: Testing and tooling may need to simulate live mutations even when runtime hot reload is
  /// disabled. Provide a narrow escape hatch without exposing broadly.
  ///
  /// 中文：測試/工具情境可繞過 hot_reload 限制強制更新；避免在核心流程濫用。
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
