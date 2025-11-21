use std::net::{SocketAddr, ToSocketAddrs};

use crate::cofg::config::{Cofg, CofgAddrs};

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
