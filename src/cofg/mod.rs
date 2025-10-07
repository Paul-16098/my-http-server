pub(crate) mod cli;
pub(crate) mod config;

/// Merge CLI overrides into loaded config.
///
/// WHY: Preserve file-based config as baseline; explicit CLI flags have higher precedence.
/// 中文：以設定檔為基礎，命令列參數覆寫對應欄位。
pub(crate) fn build_config_from_cli(mut s: config::Cofg, cli: &cli::Args) -> config::Cofg {
  match (&cli.ip, cli.port) {
    (None, None) => (),
    (None, Some(port)) => {
      s.addrs.port = port;
    }
    (Some(ip), None) => {
      s.addrs.ip = ip.to_string();
    }
    (Some(_), Some(_)) => {
      s.addrs = cli.into();
    }
  }
  s
}
