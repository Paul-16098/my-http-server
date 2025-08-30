//! cofg main
use std::collections::HashMap;

use nest_struct::nest_struct;

pub(crate) const BULID_COFG: &str = include_str!("cofg.yaml");

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
  pub(crate) watch: bool,
  pub(crate) templating_value: Option<HashMap<String, String>>,
}
impl std::fmt::Display for CofgAddrs {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.ip, self.port)
  }
}

impl Default for Cofg {
  fn default() -> Self {
    config::Config
      ::builder()
      .add_source(config::File::from_str(BULID_COFG, config::FileFormat::Yaml))
      .build()
      .unwrap()
      .try_deserialize::<Self>()
      .unwrap()
  }
}
