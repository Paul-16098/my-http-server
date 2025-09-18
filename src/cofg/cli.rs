use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
pub(crate) struct Args {
  #[arg(long)]
  pub(crate) ip: Option<String>,
  #[arg(long)]
  pub(crate) port: Option<u16>,
}

impl From<&Args> for super::CofgAddrs {
  fn from(val: &Args) -> Self {
    super::CofgAddrs {
      ip: val.ip.clone().unwrap(),
      port: val.port.unwrap(),
    }
  }
}
