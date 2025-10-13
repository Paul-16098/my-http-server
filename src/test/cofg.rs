//! test for cofg.rs

use crate::cofg::config;

#[test]
/// test for CofgAddrs Display
fn cofg_addrs_to_string() {
  assert_eq!(
    (config::CofgAddrs {
      ip: "127.0.0.1".to_string(),
      port: 8080,
    }).to_string(),
    "127.0.0.1:8080"
  )
}
#[test]
fn cofg_default() {
  config::Cofg::default();
}

#[test]
fn ip_filter_config_structure() {
  let cofg = config::Cofg::default();
  assert!(!cofg.middleware.ip_filter.enable);
  assert_eq!(cofg.middleware.ip_filter.allow, None);
  assert_eq!(cofg.middleware.ip_filter.block, None);
}
