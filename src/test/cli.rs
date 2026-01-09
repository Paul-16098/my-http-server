use crate::cofg::{cli::Args, config::CofgAddrs};

#[test]
fn test_try_from_args_with_valid_ip_and_port() {
    let args = Args {
        ip: Some("127.0.0.1".to_string()),
        port: Some(8080),
        tls_cert: None,
        tls_key: None,
        config_path: None,
        no_config: false,
        public_path: None,
        root_dir: None,
        hot_reload: None,
    };

    let result = CofgAddrs::try_from(&args);
    assert!(result.is_ok());
    let cofg_addrs = result.unwrap();
    assert_eq!(cofg_addrs.ip, "127.0.0.1");
    assert_eq!(cofg_addrs.port, 8080);
}

#[test]
fn test_try_from_args_with_missing_ip() {
    let args = Args {
        ip: None,
        port: Some(8080),
        tls_cert: None,
        tls_key: None,
        config_path: None,
        no_config: false,
        public_path: None,
        root_dir: None,
        hot_reload: None,
    };

    let result = CofgAddrs::try_from(&args);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Other error: ip or port is none".to_string()
    );
}

#[test]
fn test_try_from_args_with_missing_port() {
    let args = Args {
        ip: Some("127.0.0.1".to_string()),
        port: None,
        tls_cert: None,
        tls_key: None,
        config_path: None,
        no_config: false,
        public_path: None,
        root_dir: None,
        hot_reload: None,
    };

    let result = CofgAddrs::try_from(&args);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Other error: ip or port is none".to_string()
    );
}

#[test]
fn test_try_from_args_with_missing_ip_and_port() {
    let args = Args {
        ip: None,
        port: None,
        tls_cert: None,
        tls_key: None,
        config_path: None,
        no_config: false,
        public_path: None,
        root_dir: None,
        hot_reload: None,
    };

    let result = CofgAddrs::try_from(&args);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Other error: ip or port is none".to_string()
    );
}

#[test]
fn test_try_from_owned_args_with_valid_ip_and_port() {
    let args = Args {
        ip: Some("192.168.1.1".to_string()),
        port: Some(3000),
        tls_cert: None,
        tls_key: None,
        config_path: None,
        no_config: false,
        public_path: None,
        root_dir: None,
        hot_reload: None,
    };

    let result = CofgAddrs::try_from(args);
    assert!(result.is_ok());
    let cofg_addrs = result.unwrap();
    assert_eq!(cofg_addrs.ip, "192.168.1.1");
    assert_eq!(cofg_addrs.port, 3000);
}

#[test]
fn test_config_file_path_with_no_config() {
    let args = Args {
        ip: None,
        port: None,
        tls_cert: None,
        tls_key: None,
        config_path: None,
        no_config: true,
        public_path: None,
        root_dir: None,
        hot_reload: None,
    };
    assert_eq!(args.config_file_path(), None);
}

#[test]
fn test_config_file_path_with_custom_path() {
    let args = Args {
        ip: None,
        port: None,
        tls_cert: None,
        tls_key: None,
        config_path: Some("/etc/myhttp/config.yaml".to_string()),
        no_config: false,
        public_path: None,
        root_dir: None,
        hot_reload: None,
    };
    assert_eq!(args.config_file_path(), Some("/etc/myhttp/config.yaml"));
}

#[test]
fn test_config_file_path_default() {
    let args = Args {
        ip: None,
        port: None,
        tls_cert: None,
        tls_key: None,
        config_path: None,
        no_config: false,
        public_path: None,
        root_dir: None,
        hot_reload: None,
    };
    assert_eq!(args.config_file_path(), Some("./cofg.yaml"));
}

#[test]
fn test_config_file_path_no_config_overrides_path() {
    // no_config should take precedence over config_path
    let args = Args {
        ip: None,
        port: None,
        tls_cert: None,
        tls_key: None,
        config_path: Some("/custom/config.yaml".to_string()),
        no_config: true,
        public_path: None,
        root_dir: None,
        hot_reload: None,
    };
    assert_eq!(args.config_file_path(), None);
}

#[test]
fn test_cli_with_tls_args() {
    let args = Args {
        ip: Some("0.0.0.0".to_string()),
        port: Some(8443),
        tls_cert: Some("/path/to/cert.pem".to_string()),
        tls_key: Some("/path/to/key.pem".to_string()),
        config_path: None,
        no_config: true,
        public_path: None,
        root_dir: None,
        hot_reload: None,
    };

    assert_eq!(args.tls_cert, Some("/path/to/cert.pem".to_string()));
    assert_eq!(args.tls_key, Some("/path/to/key.pem".to_string()));
}

#[test]
fn test_cli_with_public_path() {
    let args = Args {
        ip: None,
        port: None,
        tls_cert: None,
        tls_key: None,
        config_path: None,
        no_config: true,
        public_path: Some("/var/www/html".to_string()),
        root_dir: None,
        hot_reload: None,
    };

    assert_eq!(args.public_path, Some("/var/www/html".to_string()));
}

#[test]
fn test_cli_with_hot_reload() {
    let args = Args {
        ip: None,
        port: None,
        tls_cert: None,
        tls_key: None,
        config_path: None,
        no_config: true,
        public_path: None,
        root_dir: None,
        hot_reload: Some(true),
    };

    assert_eq!(args.hot_reload, Some(true));
}
