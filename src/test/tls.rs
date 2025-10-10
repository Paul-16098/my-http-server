use crate::cofg::{ cli::Args, config::Cofg };

#[test]
fn tls_config_cli_override() {
  use crate::cofg::build_config_from_cli;
  let result = build_config_from_cli(
    Cofg {
      ..Default::default()
    },
    &(Args {
      ip: None,
      port: None,
      tls_cert: Some("./test_cert.pem".to_string()),
      tls_key: Some("./test_key.pem".to_string()),
    })
  ).unwrap();
  
  assert_eq!(result.tls.enable, true);
  assert_eq!(result.tls.cert, "./test_cert.pem");
  assert_eq!(result.tls.key, "./test_key.pem");
}

#[test]
fn tls_config_default_disabled() {
  let cfg = Cofg::default();
  assert_eq!(cfg.tls.enable, false);
}
