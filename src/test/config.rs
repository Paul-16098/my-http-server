use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Mutex, OnceLock};

use crate::cofg::{
    cli::Args,
    config::{Cofg, CofgAddrs},
};

/// Serialize tests that modify environment variables to prevent data races.
///
/// WHY: Environment variables are process-global, so parallel tests that set/unset
/// them can interfere with each other. This mutex ensures sequential execution.
fn with_env_lock<R>(f: impl FnOnce() -> R) -> R {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _g = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    f()
}

#[test]
fn test_cofg_addrs_display() {
    let addr = CofgAddrs {
        ip: "127.0.0.1".to_string(),
        port: 8080,
    };
    assert_eq!(addr.to_string(), "127.0.0.1:8080");
}

#[test]
fn test_cofg_addrs_to_socket_addrs() {
    let addr = CofgAddrs {
        ip: "127.0.0.1".to_string(),
        port: 8080,
    };
    let socket_addrs: Vec<SocketAddr> = addr.to_socket_addrs().unwrap().collect();
    assert_eq!(socket_addrs.len(), 1);
    assert_eq!(socket_addrs[0], "127.0.0.1:8080".parse().unwrap());
}
#[test]
fn test_cofg_configure_default_extensions() {
    let mut cofg = Cofg::default();
    cofg.toc.ext.insert("<built-in>".to_string());
    cofg.toc.ig.insert("<built-in>".to_string());
    cofg = cofg.configure_default_extensions();
    assert!(cofg.toc.ext.contains("html"));
    assert!(cofg.toc.ig.contains("node_modules"));
}

#[test]
fn test_layered_config_precedence_defaults_only() {
    with_env_lock(|| {
        // Ensure no environment variables from previous tests
        unsafe {
            std::env::remove_var("MYHTTP_ADDRS_IP");
            std::env::remove_var("MYHTTP_ADDRS_PORT");
        }

        // Test with --no-config flag: should use only built-in defaults
        let cli = Args {
            ip: None,
            port: None,
            tls_cert: None,
            tls_key: None,
            config_path: None,
            no_config: true,
            public_path: None,
            root_dir: None,
            page_404_path: None,
            hbs_path: None,
            hot_reload: None,
        };

        let config = Cofg::new_layered(&cli).unwrap();

        // Should have default values from BUILD_COFG
        assert_eq!(config.addrs.ip, "localhost");
        assert_eq!(config.addrs.port, 8080);
        assert_eq!(config.public_path, "./");
    });
}

#[test]
fn test_layered_config_cli_overrides() {
    with_env_lock(|| {
        // Ensure no environment variables from previous tests
        unsafe {
            std::env::remove_var("MYHTTP_ADDRS_IP");
            std::env::remove_var("MYHTTP_ADDRS_PORT");
        }

        // Test that CLI args override defaults
        let cli = Args {
            ip: Some("0.0.0.0".to_string()),
            port: Some(9090),
            tls_cert: None,
            tls_key: None,
            config_path: None,
            no_config: true,
            public_path: Some("./content/".to_string()),
            root_dir: None,
            page_404_path: None,
            hbs_path: None,
            hot_reload: Some(true),
        };

        let config = Cofg::new_layered(&cli).unwrap();

        // CLI values should override defaults
        assert_eq!(config.addrs.ip, "0.0.0.0");
        assert_eq!(config.addrs.port, 9090);
        assert_eq!(config.public_path, "./content/");
        assert!(config.templating.hot_reload);
    });
}

#[test]
fn test_layered_config_tls_requires_both() {
    with_env_lock(|| {
        // Ensure no environment variables from previous tests
        unsafe {
            std::env::remove_var("MYHTTP_ADDRS_IP");
            std::env::remove_var("MYHTTP_ADDRS_PORT");
        }

        // Test that TLS is only enabled when both cert and key are provided
        let cli_cert_only = Args {
            ip: None,
            port: None,
            tls_cert: Some("cert.pem".to_string()),
            tls_key: None,
            config_path: None,
            no_config: true,
            public_path: None,
            root_dir: None,
            page_404_path: None,
            hbs_path: None,
            hot_reload: None,
        };

        let config = Cofg::new_layered(&cli_cert_only).unwrap();
        assert!(!config.tls.enable); // TLS not enabled with cert only

        let cli_both = Args {
            ip: None,
            port: None,
            tls_cert: Some("cert.pem".to_string()),
            tls_key: Some("key.pem".to_string()),
            config_path: None,
            no_config: true,
            public_path: None,
            root_dir: None,
            page_404_path: None,
            hbs_path: None,
            hot_reload: None,
        };

        let config = Cofg::new_layered(&cli_both).unwrap();
        assert!(config.tls.enable); // TLS enabled with both
        assert_eq!(config.tls.cert, "cert.pem");
        assert_eq!(config.tls.key, "key.pem");
    });
}

#[test]
fn test_layered_config_env_variables() {
    with_env_lock(|| {
        // Set environment variables (MYHTTP_ prefix + single _ separator)
        unsafe {
            std::env::set_var("MYHTTP_ADDRS_IP", "192.168.1.1");
            std::env::set_var("MYHTTP_ADDRS_PORT", "3000");
        }

        let cli = Args {
            ip: None,
            port: None,
            tls_cert: None,
            tls_key: None,
            config_path: None,
            no_config: true,
            public_path: None,
            root_dir: None,
            page_404_path: None,
            hbs_path: None,
            hot_reload: None,
        };

        let config = Cofg::new_layered(&cli).unwrap();

        // Environment variables should override defaults
        assert_eq!(config.addrs.ip, "192.168.1.1");
        assert_eq!(config.addrs.port, 3000);

        // Clean up
        unsafe {
            std::env::remove_var("MYHTTP_ADDRS_IP");
            std::env::remove_var("MYHTTP_ADDRS_PORT");
        }
    });
}

#[test]
fn test_layered_config_cli_overrides_env() {
    with_env_lock(|| {
        // Set environment variables (MYHTTP_ prefix + single _ separator)
        unsafe {
            std::env::set_var("MYHTTP_ADDRS_PORT", "3000");
        }

        let cli = Args {
            ip: None,
            port: Some(4000), // CLI should win
            tls_cert: None,
            tls_key: None,
            config_path: None,
            no_config: true,
            public_path: None,
            root_dir: None,
            page_404_path: None,
            hbs_path: None,
            hot_reload: None,
        };

        let config = Cofg::new_layered(&cli).unwrap();

        // CLI should override environment
        assert_eq!(config.addrs.port, 4000);

        // Clean up
        unsafe {
            std::env::remove_var("MYHTTP_ADDRS_PORT");
        }
    });
}

#[test]
fn test_config_page_404_path_and_hbs_path() {
    with_env_lock(|| {
        // Test that config loads with default paths for 404 and template
        let cli = Args {
            ip: None,
            port: None,
            tls_cert: None,
            tls_key: None,
            config_path: None,
            no_config: true,
            public_path: None,
            root_dir: None,
            page_404_path: None,
            hbs_path: None,
            hot_reload: None,
        };

        let config = Cofg::new_layered(&cli).unwrap();

        // Should have default values
        assert_eq!(config.page_404_path, "./meta/404.html");
        assert_eq!(config.hbs_path, "./meta/html-t.hbs");
    });
}

#[test]
fn test_config_file_path_helper() {
    // Test --no-config flag
    let cli_no_config = Args {
        ip: None,
        port: None,
        tls_cert: None,
        tls_key: None,
        config_path: None,
        no_config: true,
        public_path: None,
        root_dir: None,
        page_404_path: None,
        hbs_path: None,
        hot_reload: None,
    };
    assert_eq!(cli_no_config.config_file_path(), None);

    // Test custom config path
    let cli_custom = Args {
        ip: None,
        port: None,
        tls_cert: None,
        tls_key: None,
        config_path: Some("/custom/config.yaml".to_string()),
        no_config: false,
        public_path: None,
        root_dir: None,
        page_404_path: None,
        hbs_path: None,
        hot_reload: None,
    };
    assert_eq!(cli_custom.config_file_path(), Some("/custom/config.yaml"));

    // Test default path
    let cli_default = Args {
        ip: None,
        port: None,
        tls_cert: None,
        tls_key: None,
        config_path: None,
        no_config: false,
        public_path: None,
        root_dir: None,
        page_404_path: None,
        hbs_path: None,
        hot_reload: None,
    };
    assert_eq!(cli_default.config_file_path(), Some("./cofg.yaml"));
}

#[test]
fn test_xdg_config_path_exists() {
    // Test that XDG helper returns a valid path structure and exposes template/404/emojis paths
    use crate::cofg::config::Cofg;

    if let Some(xdg_paths) = Cofg::get_xdg_paths() {
        println!("XDG config path resolved to: {}", xdg_paths.cofg.display());
        assert!(xdg_paths.cofg.to_string_lossy().contains("my-http-server"));
        assert!(xdg_paths.cofg.to_string_lossy().contains("cofg.yaml"));
        assert!(xdg_paths.page_404.to_string_lossy().contains("404"));
        assert!(
            xdg_paths
                .template_hbs
                .to_string_lossy()
                .contains("html-t.hbs")
        );
        assert!(
            xdg_paths
                .emojis
                .to_string_lossy()
                .contains("emojis.json")
        );
    } else {
        println!("XDG paths could not be resolved (no home directory?)");
    }

    // Just ensure the method exists and can be called via layered config
    let _xdg_path = std::panic::catch_unwind(|| {
        let cli = Args {
            ip: None,
            port: None,
            tls_cert: None,
            tls_key: None,
            config_path: None,
            no_config: true, // Skip actual config loading
            public_path: None,
            root_dir: None,
            page_404_path: None,
            hbs_path: None,
            hot_reload: None,
        };
        Cofg::new_layered(&cli)
    });

    assert!(_xdg_path.is_ok());
}

#[test]
fn test_rate_limiting_validation() {
    // Test that invalid rate limiting values are corrected
    let cli = Args {
        ip: None,
        port: None,
        tls_cert: None,
        tls_key: None,
        config_path: None,
        no_config: true,
        public_path: None,
        root_dir: None,
        page_404_path: None,
        hbs_path: None,
        hot_reload: None,
    };

    let mut config = Cofg::new_layered(&cli).unwrap();

    // Force invalid values
    config.middleware.rate_limiting.burst_size = 0;
    config.middleware.rate_limiting.seconds_per_request = 0;

    // Apply overrides should fix them
    config.apply_cli_overrides(&cli).unwrap();

    assert_eq!(config.middleware.rate_limiting.burst_size, 1);
    assert_eq!(config.middleware.rate_limiting.seconds_per_request, 1);
}
