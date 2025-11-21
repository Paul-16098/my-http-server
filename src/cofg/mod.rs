use crate::error::AppResult;

pub(crate) mod cli;
pub(crate) mod config;

/// Merge CLI overrides into loaded config.
///
/// WHY: Preserve file-based config as baseline; explicit CLI flags have higher precedence.
/// WHY: Configuration file as base; CLI arguments override corresponding fields.
pub(crate) fn build_config_from_cli(
    mut s: config::Cofg,
    cli: &cli::Args,
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
    if let Some(dir) = &cli.root_dir {
        // println!("Overriding root_dir to {}", dir);
        std::env::set_current_dir(dir)?;
        let mut new_cli = cli.clone();
        new_cli.root_dir = None;

        let new_cofg = build_config_from_cli(config::Cofg::new(), &new_cli);
        // println!("New cofg from root_dir: {:?}", new_cofg);

        return new_cofg;
    }

    Ok(s)
}
