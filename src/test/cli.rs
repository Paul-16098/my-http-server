use crate::cofg::{ cli::Args, config::CofgAddrs };

#[test]
fn test_try_from_args_with_valid_ip_and_port() {
  let args = Args {
    ip: Some("127.0.0.1".to_string()),
    port: Some(8080),
    tls_cert: None,
    tls_key: None,
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
  };

  let result = CofgAddrs::try_from(&args);
  assert!(result.is_err());
  assert_eq!(result.unwrap_err().to_string(), "Other error: ip or port is none".to_string());
}

#[test]
fn test_try_from_args_with_missing_port() {
  let args = Args {
    ip: Some("127.0.0.1".to_string()),
    port: None,
    tls_cert: None,
    tls_key: None,
  };

  let result = CofgAddrs::try_from(&args);
  assert!(result.is_err());
  assert_eq!(result.unwrap_err().to_string(), "Other error: ip or port is none".to_string());
}

#[test]
fn test_try_from_args_with_missing_ip_and_port() {
  let args = Args {
    ip: None,
    port: None,
    tls_cert: None,
    tls_key: None,
  };

  let result = CofgAddrs::try_from(&args);
  assert!(result.is_err());
  assert_eq!(result.unwrap_err().to_string(), "Other error: ip or port is none".to_string());
}

#[test]
fn test_try_from_owned_args_with_valid_ip_and_port() {
  let args = Args {
    ip: Some("192.168.1.1".to_string()),
    port: Some(3000),
    tls_cert: None,
    tls_key: None,
  };

  let result = CofgAddrs::try_from(args);
  assert!(result.is_ok());
  let cofg_addrs = result.unwrap();
  assert_eq!(cofg_addrs.ip, "192.168.1.1");
  assert_eq!(cofg_addrs.port, 3000);
}
