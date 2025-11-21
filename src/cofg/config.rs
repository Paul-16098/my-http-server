//! Configuration (Cofg)
//!
//! WHY: Centralized runtime configuration cached in a global `OnceCell<RwLock<_>>` so hot paths
//! (HTTP request handling & markdown rendering) avoid disk IO / deserialization cost. A reload is
//! only attempted when caller explicitly asks (`get(true)`) AND hot-reload is enabled. This keeps
//! the steady-state fast while still offering a development-friendly live tweaking mode.

use nest_struct::nest_struct;
use once_cell::sync::OnceCell;
use std::{collections::HashSet, sync::RwLock};

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
        /// Whitelisted IPs
        pub(crate) allow: Option<Vec<String>>,
        /// Blacklisted IPs
        pub(crate) block: Option<Vec<String>>
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
static GLOBAL_COFG: OnceCell<RwLock<Cofg>> = OnceCell::new();

impl Default for Cofg {
    fn default() -> Self {
        Cofg::new_from_str(BUILD_COFG)
    }
}

impl Cofg {
    /// Load configuration from disk, creating a default file if it doesn't exist.
    /// WHY: Supports scenarios like admin commands or live reload utilities.
    pub fn load_from_disk() -> Self {
        if !std::path::Path::new("./cofg.yaml").exists() {
            println!("write default cofg");
            std::fs::write("./cofg.yaml", BUILD_COFG).unwrap();
        }
        Self::new_from_source(config::File::with_name("./cofg.yaml"))
    }
    // Accept any owned source type that implements `config::Source`.
    // This avoids passing a reference to a trait object which doesn't satisfy
    // the builder's `add_source<T: Source + Send + Sync + 'static>(T)` bound.
    pub fn new_from_source<T>(source: T) -> Self
    where
        T: config::Source + Send + Sync + 'static,
    {
        config::Config::builder()
            .add_source(source)
            .build()
            .unwrap()
            .try_deserialize::<Self>()
            .unwrap()
            .configure_default_extensions()
    }

    /// new from yaml string
    pub fn new_from_str(date_str: &str) -> Self {
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

    pub(crate) fn new() -> Self {
        Self::get(true)
    }

    /// if `reload` is true, reload config from disk
    pub(crate) fn get(reload: bool) -> Self {
        let cell = GLOBAL_COFG.get_or_init(|| RwLock::new(Self::load_from_disk()));

        if reload && let Ok(mut guard) = cell.write() {
            *guard = Self::load_from_disk();
        }

        cell.read().map(|g| g.clone()).unwrap_or_default()
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
