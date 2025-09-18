use crate::cofg::{ cli::Args, Cofg, CofgAddrs };

#[test]
fn form_test() {
  assert_eq!(
    CofgAddrs::from(&(Args { ip: Some("127.0.0.1".to_string()), port: Some(6426) })),
    CofgAddrs {
      ip: "127.0.0.1".to_string(),
      port: 6426,
    }
  )
}
#[test]
fn build_config_from_cli_test() {
  use crate::build_config_from_cli;
  assert_eq!(
    build_config_from_cli(
      Cofg {
        ..Default::default()
      },
      &(Args { ip: Some("127.0.0.1".to_string()), port: Some(6426) })
    ),
    Cofg {
      addrs: CofgAddrs { ip: "127.0.0.1".to_string(), port: 6426 },
      ..Default::default()
    }
  )
}
