//! CLI argument parsing for overriding config
//!
//! WHY: Allow quick overrides (ip/port) without editing config file. Keep the surface small and
//! explicit to avoid accidental drift from file-based defaults.
//!
//! 中文：提供最小集合的命令列參數覆寫設定檔（IP/Port），便於臨時調整。

use clap::Parser;

use crate::error::AppError;

#[derive(Parser, Debug, Clone)]
#[command(version = crate::VERSION.to_string())]
pub(crate) struct Args {
    #[arg(long)]
    /// IP address to bind the server to
    pub(crate) ip: Option<String>,
    #[arg(long)]
    /// Port number to bind the server to
    pub(crate) port: Option<u16>,
    #[arg(long)]
    /// Path to TLS certificate file (PEM format)
    pub(crate) tls_cert: Option<String>,
    #[arg(long)]
    /// Path to TLS private key file (PEM format)
    pub(crate) tls_key: Option<String>,
    #[arg()]
    /// Root directory for execution context
    /// config, templates, static files, etc. will be resolved relative to this path.
    pub(crate) root_dir: Option<String>,
    #[arg(long, short, default_value_t = false)]
    /// do not load configuration file from disk, only use defaults and CLI args
    pub(crate) no_config: bool,
}

impl TryFrom<&Args> for super::config::CofgAddrs {
    type Error = AppError;

    fn try_from(val: &Args) -> Result<Self, Self::Error> {
        if let (Some(ip), Some(port)) = (&val.ip, val.port) {
            Ok(Self {
                ip: ip.clone(),
                port,
            })
        } else {
            Err(AppError::OtherError("ip or port is none".to_string()))
        }
    }
}
impl TryFrom<Args> for super::config::CofgAddrs {
    type Error = AppError;

    fn try_from(val: Args) -> Result<Self, Self::Error> {
        if let (Some(ip), Some(port)) = (&val.ip, val.port) {
            Ok(Self {
                ip: ip.clone(),
                port,
            })
        } else {
            Err(AppError::OtherError("ip or port is none".to_string()))
        }
    }
}
