//! test for cofg.rs
#[test]
/// test for CofgAddrs Display
fn cofg_addrs_to_string() {
  assert_eq!(
    (crate::cofg::CofgAddrs {
      ip: "127.0.0.1".to_string(),
      port: 8080,
    }).to_string(),
    "127.0.0.1:8080"
  )
}
