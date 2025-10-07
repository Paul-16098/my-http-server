//! CLI argument parsing for overriding config
//!
//! WHY: Allow quick overrides (ip/port) without editing config file. Keep the surface small and
//! explicit to avoid accidental drift from file-based defaults.
//!
//! 中文：提供最小集合的命令列參數覆寫設定檔（IP/Port），便於臨時調整。

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
pub(crate) struct Args {
  #[arg(long)]
  pub(crate) ip: Option<String>,
  #[arg(long)]
  pub(crate) port: Option<u16>,
}

impl From<&Args> for super::config::CofgAddrs {
  fn from(val: &Args) -> Self {
    // Invariant: when converting, both fields should be present. Callers ensure this by matching
    // on `(Some, Some)` only. Debug assertions help catch misuse in tests/development.
    debug_assert!(val.ip.is_some() && val.port.is_some());
    super::config::CofgAddrs {
      ip: val.ip.clone().unwrap(),
      port: val.port.unwrap(),
    }
  }
}
