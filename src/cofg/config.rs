//! Configuration (Cofg)
//!
//! WHY: Centralized runtime configuration with layered precedence:
//! 1. Built-in defaults (embedded cofg.yaml)
//! 2. Config file (./cofg.yaml or --config-path)
//! 3. Environment variables (MYHTTP_* prefix)
//! 4. CLI arguments (highest priority)
//!
//! Global caching via `OnceCell<RwLock<_>>` ensures hot paths (HTTP request handling & markdown
//! rendering) avoid disk IO/deserialization cost. Hot reload respects the precedence chain and
//! only reloads the file layer when `templating.hot_reload=true`.
//!
//! 中文：分層配置系統，優先級：內建預設→配置檔→環境變數→CLI參數。全局快取避免熱路徑IO開銷。

use log::{debug, error, warn};
use nest_struct::nest_struct;
use std::{
    collections::HashSet,
    sync::{OnceLock, RwLock},
};

use crate::error::AppResult;

pub(crate) const BUILD_COFG: &str = include_str!("cofg.yaml");

#[nest_struct]
#[derive(PartialEq, Clone, Debug, serde::Deserialize)]
pub(crate) struct Cofg {
    pub(crate) addrs: nest! {
      /// Server IP address (e.g., 127.0.0.1)
      pub(crate) ip: String,
      /// Server port (e.g., 80, 8080)
      pub(crate) port: u16,
    },
    pub(crate) tls: nest! {
      /// Enable TLS/HTTPS
      pub(crate) enable: bool,
      /// Path to TLS certificate file (PEM format)
      pub(crate) cert: String,
      /// Path to TLS private key file (PEM format)
      pub(crate) key: String,
    },
    pub(crate) middleware: nest! {
      /// Enable NormalizePath middleware
      pub(crate) normalize_path: bool,
      /// Enable Compress middleware
      pub(crate) compress: bool,
      pub(crate) logger: nest! {
        /// Enable request logging
        pub(crate) enabling: bool,
        /// Logger output format
        pub(crate) format: String
      },
      pub(crate) http_base_authentication: nest! {
        /// Enable HTTP Basic Authentication
        pub(crate) enable: bool,
        /// List of users for authentication
        pub(crate) users: Option<Vec<nest! {
          /// Username
          pub(crate) name: String,
          /// Password (optional)
          pub(crate) passwords: Option<String>,
          /// Allowed paths for this user
          pub(crate) allow: Option<Vec<String>>,
          /// Disallowed paths for this user
          pub(crate) disallow: Option<Vec<String>>
        }>>
      },
      pub(crate) ip_filter: nest! {
        /// Enable IP filtering
        pub(crate) enable: bool,
        pub(crate) rules: Vec<nest!{
          pub(crate) limit_to: Vec<String>,
          /// Whitelisted IPs
          pub(crate) allow: Option<Vec<String>>,
          /// Blacklisted IPs
          pub(crate) block: Option<Vec<String>>
        }>,
      },
      pub(crate) rate_limiting: nest! {
        /// Enable rate limiting
        pub(crate) enable: bool,
        /// Minimum seconds between requests
        pub(crate) seconds_per_request: u64,
        /// Maximum burst size for requests
        pub(crate) burst_size: u32
      }
    },
    #[cfg(feature = "api")]
    pub(crate) api: nest! {
      pub(crate) enable: bool,
      pub(crate) allow_edit: bool
    },
    pub(crate) templating: nest! {
      /// Custom template values
      pub(crate) value: Option<Vec<String>>,
      /// Enable hot-reloading of templates
      pub(crate) hot_reload: bool
    },
    pub(crate) toc: nest! {
      /// File extensions to include in TOC generation
      pub(crate) ext: HashSet<String>,
      /// Directories to ignore in TOC generation
      pub(crate) ig: HashSet<String>
    },
    /// Path to the public directory
    pub(crate) public_path: String,
}

// global cached config; allow refresh when hot_reload = true
// Global cached config with CLI args for proper layered reload
struct GlobalConfig {
    config: Cofg,
    cli_args: Option<super::cli::Args>,
}

static GLOBAL_COFG: OnceLock<RwLock<GlobalConfig>> = OnceLock::new();

impl Default for Cofg {
    fn default() -> Self {
        match Cofg::new_from_str(BUILD_COFG) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to load default configuration: {}", e);
                panic!("Failed to load default configuration: {}", e);
            }
        }
    }
}

impl Cofg {
    /// Load configuration from disk.
    /// Build layered configuration from CLI arguments.
    ///
    /// This is the primary entry point for loading configuration with full precedence chain:
    /// 1. Built-in defaults (BUILD_COFG)
    /// 2. Config file (if not --no-config)
    /// 3. Environment variables (MYHTTP_* prefix)
    /// 4. CLI overrides (highest priority)
    ///
    /// WHY: Explicit precedence chain makes config behavior predictable and testable.
    pub fn new_layered(cli: &super::cli::Args) -> AppResult<Self> {
        let mut builder = config::Config::builder()
            // Layer 1: Built-in defaults
            .add_source(config::File::from_str(BUILD_COFG, config::FileFormat::Yaml));

        // Layer 2: Config file (unless --no-config)
        if let Some(config_path) = cli.config_file_path() {
            let path = std::path::Path::new(config_path);
            if path.exists() {
                debug!("Loading config from: {}", config_path);
                builder = builder.add_source(config::File::from(path));
            } else if !cli.no_config {
                // Only warn if user didn't explicitly skip config
                warn!("Config file not found: {}, using defaults", config_path);
            }
        }

        // Layer 3: Environment variables with MYHTTP_ prefix
        // Map nested config like "addrs.ip" to MYHTTP_ADDRS_IP (separator="_")
        builder = builder.add_source(
            config::Environment::with_prefix("MYHTTP")
                .separator("_")
                .try_parsing(true),
        );

        let mut cfg = builder
            .build()?
            .try_deserialize::<Self>()?
            .configure_default_extensions();

        // Layer 4: CLI overrides (highest priority)
        cfg.apply_cli_overrides(cli)?;

        Ok(cfg)
    }

    /// Apply CLI argument overrides to the configuration.
    ///
    /// WHY: Keep CLI override logic centralized and explicit.
    pub(crate) fn apply_cli_overrides(&mut self, cli: &super::cli::Args) -> AppResult<()> {
        // Server binding
        if let Some(ref ip) = cli.ip {
            self.addrs.ip = ip.clone();
        }
        if let Some(port) = cli.port {
            self.addrs.port = port;
        }

        // TLS configuration (require both cert and key)
        if let (Some(cert), Some(key)) = (&cli.tls_cert, &cli.tls_key) {
            self.tls.cert = cert.clone();
            self.tls.key = key.clone();
            self.tls.enable = true;
        }

        // Public path
        if let Some(ref path) = cli.public_path {
            self.public_path = path.clone();
        }

        // Hot reload
        if let Some(hot_reload) = cli.hot_reload {
            self.templating.hot_reload = hot_reload;
        }

        // Validation: ensure rate limiting values are sane
        if self.middleware.rate_limiting.burst_size == 0 {
            warn!("burst_size of 0 is invalid; setting to 1");
            self.middleware.rate_limiting.burst_size = 1;
        }
        if self.middleware.rate_limiting.seconds_per_request == 0 {
            warn!("seconds_per_request of 0 is invalid; setting to 1");
            self.middleware.rate_limiting.seconds_per_request = 1;
        }

        Ok(())
    }
    /// WHY: Supports scenarios like admin commands or live reload utilities.
    pub fn load_from_disk() -> AppResult<Self> {
        Self::new_from_source(config::File::with_name("./cofg.yaml"))
    }

    /// Load configuration from disk, creating a default file if it doesn't exist.
    pub fn load_from_disk_or_init() -> AppResult<Self> {
        if !std::path::Path::new("./cofg.yaml").exists() {
            debug!("write default cofg");
            if let Err(e) = std::fs::write("./cofg.yaml", BUILD_COFG) {
                warn!("Failed to write default configuration file: {}", e)
            };
        }
        Self::load_from_disk()
    }
    // Accept any owned source type that implements `config::Source`.
    // This avoids passing a reference to a trait object which doesn't satisfy
    // the builder's `add_source<T: Source + Send + Sync + 'static>(T)` bound.
    pub fn new_from_source<T>(source: T) -> AppResult<Self>
    where
        T: config::Source + Send + Sync + 'static,
    {
        Ok(config::Config::builder()
            .add_source(source)
            .build()?
            .try_deserialize::<Self>()?
            .configure_default_extensions())
    }

    /// new from yaml string
    pub fn new_from_str(date_str: &str) -> AppResult<Self> {
        Self::new_from_source(config::File::from_str(date_str, config::FileFormat::Yaml))
    }
    /// Configure default extensions for TOC generation.
    pub(crate) fn configure_default_extensions(mut self) -> Self {
        if self.toc.ext.contains("<build-in>") {
            self.toc.ext.remove("<build-in>");
            self.toc.ext.extend(
                ["html", "md", "pdf", "txt", "png"]
                    .into_iter()
                    .map(String::from),
            );
        }
        if self.toc.ig.contains("<build-in>") {
            self.toc.ig.remove("<build-in>");
            self.toc
                .ig
                .extend(["node_modules"].into_iter().map(String::from));
        }
        self
    }

    // #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::get(false)
    }

    /// if `reload` is true, reload config from disk
    pub fn get(force_reload: bool) -> Self {
        Self::get_global(force_reload).unwrap_or_else(|e| {
            error!("Failed to get global configuration: {}", e);
            Self::default()
        })
    }

    /// Get global configuration with optional forced reload.
    ///
    /// If `force_reload=true`, reloads config from file layer only (respects CLI/env overrides).
    /// Hot reload is only allowed when `templating.hot_reload=true` in the config.
    ///
    /// WHY: Enforce hot reload guard to prevent accidental reloads in production.
    pub(crate) fn get_global(force_reload: bool) -> AppResult<Self> {
        let cell = GLOBAL_COFG.get_or_init(|| {
            debug!("Initializing global configuration");
            let config = Self::load_from_disk_or_init().unwrap_or_else(|e| {
                warn!("Failed to load configuration from disk: {}", e);
                Self::default()
            });
            RwLock::new(GlobalConfig {
                config,
                cli_args: None,
            })
        });

        // Attempt reload if requested
        if force_reload {
            if let Ok(guard) = cell.read() {
                // Check if hot reload is enabled before allowing reload
                if !guard.config.templating.hot_reload {
                    debug!("Hot reload requested but not enabled in config");
                    return Ok(guard.config.clone());
                }
            }

            if let Ok(mut guard) = cell.write() {
                // Reload with original CLI args if available
                let new_config = if let Some(ref cli_args) = guard.cli_args {
                    debug!("Reloading config with CLI args");
                    Self::new_layered(cli_args)?
                } else {
                    debug!("Reloading config from disk");
                    Self::load_from_disk_or_init()?
                };
                guard.config = new_config;
            }
        }

        Ok(cell.read().map(|g| g.config.clone()).unwrap_or_default())
    }

    /// Initialize global configuration with CLI arguments.
    ///
    /// This should be called once at startup to establish the config with full precedence chain.
    ///
    /// WHY: Store CLI args for proper reload behavior that respects original overrides.
    pub fn init_global(cli: &super::cli::Args) -> AppResult<Self> {
        let config = Self::new_layered(cli)?;

        let cell = GLOBAL_COFG.get_or_init(|| {
            RwLock::new(GlobalConfig {
                config: config.clone(),
                cli_args: Some(cli.clone()),
            })
        });

        // Update if already initialized (e.g., from tests)
        if let Ok(mut guard) = cell.write() {
            guard.config = config.clone();
            guard.cli_args = Some(cli.clone());
        }

        Ok(config)
    }
}

impl std::fmt::Display for CofgAddrs {
    /// Format the address as `IP:Port`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

impl std::net::ToSocketAddrs for CofgAddrs {
    type Iter = std::vec::IntoIter<std::net::SocketAddr>;

    /// Convert the address to a socket address.
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        std::net::ToSocketAddrs::to_socket_addrs(&(self.ip.as_str(), self.port))
    }
}
