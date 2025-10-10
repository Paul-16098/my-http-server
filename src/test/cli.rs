use crate::cofg::{ cli::Args, config::{ Cofg, CofgAddrs } };

#[test]
fn form_test() {
  assert_eq!(
    std::convert::TryInto::<CofgAddrs>
      ::try_into(Args {
        ip: Some("127.0.0.1".to_string()),
        port: Some(6426),
        tls_cert: None,
        tls_key: None,
      })
      .unwrap(),
    CofgAddrs {
      ip: "127.0.0.1".to_string(),
      port: 6426,
    }
  )
}
#[test]
fn build_config_from_cli_test() {
  use crate::cofg::build_config_from_cli;
  assert_eq!(
    build_config_from_cli(
      Cofg {
        ..Default::default()
      },
      &(Args {
        ip: Some("127.0.0.1".to_string()),
        port: Some(6426),
        tls_cert: None,
        tls_key: None,
      })
    ).unwrap(),
    Cofg {
      addrs: CofgAddrs {
        ip: "127.0.0.1".to_string(),
        port: 6426,
      },
      ..Default::default()
    }
  )
}
