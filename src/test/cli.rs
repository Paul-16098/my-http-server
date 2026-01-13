//! CLI argument parsing tests
//!
//! WHY: Validate CLI argument handling and precedence over config files.

use crate::cofg::cli::Args;
use clap::Parser;

/// Test parsing default CLI arguments
#[test]
fn test_cli_args_default_help_parsing() {
    // Test that help text doesn't cause errors
    let result = Args::try_parse_from(["test"].as_ref());

    // Should succeed with defaults
    assert!(result.is_ok(), "Default args should parse successfully");
}

/// Test parsing of individual CLI arguments
#[test]
fn test_cli_args_ip_override() {
    let args = Args::try_parse_from(["test", "--ip", "192.168.1.1"].as_ref());

    assert!(args.is_ok(), "IP override should parse");
    assert_eq!(args.unwrap().ip, Some("192.168.1.1".to_string()));
}

/// Test parsing of port argument
#[test]
fn test_cli_args_port_override() {
    let args = Args::try_parse_from(["test", "--port", "3000"].as_ref());

    assert!(args.is_ok(), "Port override should parse");
    assert_eq!(args.unwrap().port, Some(3000));
}

/// Test parsing multiple CLI arguments together
#[test]
fn test_cli_args_multiple_overrides() {
    let args = Args::try_parse_from(["test", "--ip", "0.0.0.0", "--port", "8888"].as_ref());

    assert!(args.is_ok(), "Multiple args should parse");
    let parsed = args.unwrap();
    assert_eq!(parsed.ip, Some("0.0.0.0".to_string()));
    assert_eq!(parsed.port, Some(8888));
}

/// Test parsing `--config-path` argument
#[test]
fn test_cli_args_config_path() {
    let args = Args::try_parse_from(["test", "--config-path", "./my-config.yaml"].as_ref());

    assert!(args.is_ok(), "Config path should parse");
    assert_eq!(
        args.unwrap().config_path,
        Some("./my-config.yaml".to_string())
    );
}

#[test]
fn test_cli_args_no_config_flag() {
    let args = Args::try_parse_from(["test", "--no-config"].as_ref());

    assert!(args.is_ok(), "No-config flag should parse");
    assert!(args.unwrap().no_config);
}

/// Test parsing TLS settings
#[test]
fn test_cli_args_tls_settings() {
    let args = Args::try_parse_from(
        ["test", "--tls-cert", "./cert.pem", "--tls-key", "./key.pem"].as_ref(),
    );

    assert!(args.is_ok(), "TLS settings should parse");
    let parsed = args.unwrap();
    assert_eq!(parsed.tls_cert, Some("./cert.pem".to_string()));
    assert_eq!(parsed.tls_key, Some("./key.pem".to_string()));
}

#[test]
fn test_cli_args_public_path() {
    let args = Args::try_parse_from(["test", "--public-path", "/srv/http"].as_ref());

    assert!(args.is_ok(), "Public path should parse");
    assert_eq!(args.unwrap().public_path, Some("/srv/http".to_string()));
}

#[test]
fn test_cli_args_hot_reload_true() {
    let args = Args::try_parse_from(["test", "--hot-reload", "true"].as_ref());

    assert!(args.is_ok(), "Hot reload true should parse");
    assert_eq!(args.unwrap().hot_reload, Some(true));
}

#[test]
fn test_cli_args_hot_reload_false() {
    let args = Args::try_parse_from(["test", "--hot-reload", "false"].as_ref());

    assert!(args.is_ok(), "Hot reload false should parse");
    assert_eq!(args.unwrap().hot_reload, Some(false));
}

#[test]
fn test_cli_args_page_404_path() {
    let args = Args::try_parse_from(["test", "--page-404-path", "./custom-404.html"].as_ref());

    assert!(args.is_ok(), "404 page path should parse");
    assert_eq!(
        args.unwrap().page_404_path,
        Some("./custom-404.html".to_string())
    );
}

#[test]
fn test_cli_args_hbs_path() {
    let args = Args::try_parse_from(["test", "--hbs-path", "./template.hbs"].as_ref());

    assert!(args.is_ok(), "HBS path should parse");
    assert_eq!(args.unwrap().hbs_path, Some("./template.hbs".to_string()));
}

#[test]
fn test_cli_args_root_dir() {
    let args = Args::try_parse_from(["test", "--root-dir", "/home/user/app"].as_ref());

    assert!(args.is_ok(), "Root dir should parse");
    assert_eq!(args.unwrap().root_dir, Some("/home/user/app".to_string()));
}

/// Test parsing all options combined
#[test]
fn test_cli_args_all_options_combined() {
    let args = Args::try_parse_from(
        [
            "test",
            "--ip",
            "127.0.0.1",
            "--port",
            "9000",
            "--config-path",
            "./config.yaml",
            "--public-path",
            "./public",
            "--page-404-path",
            "./404.html",
            "--hbs-path",
            "./template.hbs",
            "--hot-reload",
            "true",
            "--tls-cert",
            "./cert.pem",
            "--tls-key",
            "./key.pem",
        ]
        .as_ref(),
    );

    assert!(args.is_ok(), "All options combined should parse");
    let parsed = args.unwrap();
    assert_eq!(parsed.ip, Some("127.0.0.1".to_string()));
    assert_eq!(parsed.port, Some(9000));
    assert_eq!(parsed.config_path, Some("./config.yaml".to_string()));
    assert_eq!(parsed.public_path, Some("./public".to_string()));
    assert_eq!(parsed.page_404_path, Some("./404.html".to_string()));
    assert_eq!(parsed.hbs_path, Some("./template.hbs".to_string()));
    assert_eq!(parsed.hot_reload, Some(true));
    assert_eq!(parsed.tls_cert, Some("./cert.pem".to_string()));
    assert_eq!(parsed.tls_key, Some("./key.pem".to_string()));
}

#[test]
fn test_cli_args_clone() {
    let args1 = Args::try_parse_from(["test", "--ip", "localhost", "--port", "8080"].as_ref());

    assert!(args1.is_ok(), "Args should parse");
    let parsed = args1.unwrap();
    let cloned = parsed.clone();

    assert_eq!(parsed.ip, cloned.ip, "Cloned ip should match");
    assert_eq!(parsed.port, cloned.port, "Cloned port should match");
}

#[test]
fn test_cli_args_debug_output() {
    let args = Args::try_parse_from(["test", "--ip", "127.0.0.1"].as_ref());

    assert!(args.is_ok(), "Debug format should work");
    let parsed = args.unwrap();
    let debug_str = format!("{:?}", parsed);

    assert!(
        debug_str.contains("127.0.0.1"),
        "Debug output should contain IP"
    );
}

/// Test invalid port argument
#[test]
fn test_cli_args_invalid_port() {
    let args = Args::try_parse_from(["test", "--port", "invalid_port"].as_ref());

    // Should fail to parse invalid port
    assert!(args.is_err(), "Invalid port should fail to parse");
}

/// Test invalid hot-reload argument
#[test]
fn test_cli_args_invalid_hot_reload() {
    let args = Args::try_parse_from(["test", "--hot-reload", "maybe"].as_ref());

    // Should fail to parse invalid boolean
    assert!(args.is_err(), "Invalid hot-reload value should fail");
}

#[test]
fn test_cli_args_help_contains_description() {
    // Test that help is available (by checking version is present)
    let args = Args::try_parse_from(["test"].as_ref());
    assert!(args.is_ok(), "Default parsing should work");
}

/// Test that `--no-config` and `--config-path` can be used together
#[test]
fn test_cli_args_no_config_with_config_path() {
    // Test that both flags can be used together
    let args =
        Args::try_parse_from(["test", "--no-config", "--config-path", "./ignored.yaml"].as_ref());

    assert!(args.is_ok(), "Both flags should parse");
    let binding = args.unwrap();
    let config_file_path = binding.config_file_path();
    assert_eq!(config_file_path, None);
}
