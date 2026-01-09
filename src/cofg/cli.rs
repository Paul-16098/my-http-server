//! CLI argument parsing for overriding config
//!
//! WHY: Allow layered configuration with clear precedence: defaults → XDG → file → environment → CLI.
//! This enables flexible deployment scenarios while maintaining explicit control.
//!
//! Configuration Precedence (lowest to highest):
//! 1. Built-in defaults (embedded cofg.yaml)
//! 2. XDG config directory ($XDG_CONFIG_HOME/my-http-server/cofg.yaml or %APPDATA%\my-http-server\cofg.yaml)
//! 3. Local config file (./cofg.yaml or --config-path)
//! 4. Environment variables (MYHTTP_* prefix, separator="_")
//! 5. CLI arguments (highest priority)
//!
//! 中文：提供分層配置系統，優先級由低到高：內建預設→XDG配置目錄→本地配置檔→環境變數→命令列參數。

use clap::Parser;

use crate::error::AppError;

#[derive(Parser, Debug, Clone)]
#[command(version = crate::VERSION.to_string())]
#[command(about = "A lightweight HTTP server for serving static files and rendering Markdown")]
pub(crate) struct Args {
    // === Server Binding ===
    #[arg(long)]
    /// IP address to bind the server to (overrides config file)
    pub(crate) ip: Option<String>,

    #[arg(long)]
    /// Port number to bind the server to (overrides config file)
    pub(crate) port: Option<u16>,

    // === TLS Configuration ===
    #[arg(long)]
    /// Path to TLS certificate file (PEM format)
    pub(crate) tls_cert: Option<String>,

    #[arg(long)]
    /// Path to TLS private key file (PEM format)
    pub(crate) tls_key: Option<String>,

    // === Config File Control ===
    #[arg(long)]
    /// Path to configuration file (default: ./cofg.yaml)
    pub(crate) config_path: Option<String>,

    #[arg(long, short = 'n', default_value_t = false)]
    /// Skip loading configuration file; use only defaults, environment variables, and CLI args
    pub(crate) no_config: bool,

    // === Path Configuration ===
    #[arg(long)]
    /// Path to public directory for serving files (overrides config file)
    pub(crate) public_path: Option<String>,

    #[arg(long)]
    /// Root directory for execution context (changes working directory before loading config)
    /// Config, templates, static files, etc. will be resolved relative to this path
    pub(crate) root_dir: Option<String>,

    // === Error Pages and Templates ===
    #[arg(long)]
    /// Path to 404 error page file (overrides config file)
    pub(crate) page_404_path: Option<String>,

    #[arg(long)]
    /// Path to HTML template file (overrides config file)
    pub(crate) hbs_path: Option<String>,

    // === Development Options ===
    #[arg(long)]
    /// Enable hot reload for templates and config (overrides config file)
    pub(crate) hot_reload: Option<bool>,
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

impl Args {
    /// Returns the effective config file path based on CLI arguments.
    ///
    /// Priority:
    /// 1. If `--no-config`: returns None (skip file loading)
    /// 2. If `--config-path <path>`: returns Some(path)
    /// 3. Default: returns Some("./cofg.yaml")
    pub(crate) fn config_file_path(&self) -> Option<&str> {
        if self.no_config {
            None
        } else if let Some(ref path) = self.config_path {
            Some(path.as_str())
        } else {
            Some("./cofg.yaml")
        }
    }
}
