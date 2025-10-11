//! CLI argument parsing for overriding config
//!
//! WHY: Allow quick overrides (ip/port) without editing config file. Keep the surface small and
//! explicit to avoid accidental drift from file-based defaults.
//!
//! 中文：提供最小集合的命令列參數覆寫設定檔（IP/Port），便於臨時調整。

use clap::Parser;

use crate::error::AppError;

#[derive(Parser, Debug)]
#[command(version = option_env!("VERSION").unwrap_or("?"))]
pub(crate) struct Args {
  #[arg(long)]
  pub(crate) ip: Option<String>,
  #[arg(long)]
  pub(crate) port: Option<u16>,
  #[arg(long)]
  pub(crate) tls_cert: Option<String>,
  #[arg(long)]
  pub(crate) tls_key: Option<String>,
}

impl TryFrom<&Args> for super::config::CofgAddrs {
  type Error = AppError;

  fn try_from(val: &Args) -> Result<Self, Self::Error> {
    if let (Some(ip), Some(port)) = (&val.ip, val.port) {
      Ok(Self { ip: ip.clone(), port })
    } else {
      Err(AppError::OtherError("ip or port is none".to_string()))
    }
  }
}
impl TryFrom<Args> for super::config::CofgAddrs {
  type Error = AppError;

  fn try_from(val: Args) -> Result<Self, Self::Error> {
    if let (Some(ip), Some(port)) = (&val.ip, val.port) {
      Ok(Self { ip: ip.clone(), port })
    } else {
      Err(AppError::OtherError("ip or port is none".to_string()))
    }
  }
}
