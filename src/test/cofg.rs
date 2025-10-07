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
