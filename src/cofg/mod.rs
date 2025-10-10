use crate::error::AppResult;

pub(crate) mod cli;
pub(crate) mod config;

/// Merge CLI overrides into loaded config.
///
/// WHY: Preserve file-based config as baseline; explicit CLI flags have higher precedence.
/// 中文：以設定檔為基礎，命令列參數覆寫對應欄位。
pub(crate) fn build_config_from_cli(
  mut s: config::Cofg,
  cli: &cli::Args
) -> AppResult<config::Cofg> {
  match (&cli.ip, cli.port) {
    (None, None) => (),
    (None, Some(port)) => {
      s.addrs.port = port;
    }
    (Some(ip), None) => {
      s.addrs.ip = ip.to_string();
    }
    (Some(_), Some(_)) => {
      s.addrs = cli.try_into()?;
    }
  }
  
  // Only enable TLS when both cert and key are provided
  if let (Some(cert), Some(key)) = (&cli.tls_cert, &cli.tls_key) {
    s.tls.cert = cert.to_string();
    s.tls.key = key.to_string();
    s.tls.enable = true;
  }
  
  Ok(s)
}
