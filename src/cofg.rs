use nest_struct::nest_struct;

#[nest_struct]
#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct Cofg {
  /// like: 127.0.0.1
  pub(crate) ip: String,
  /// like: 80, 8080
  pub(crate) port: u16,
}

impl Default for Cofg {
  fn default() -> Self {
    config::Config
      ::builder()
      .add_source(config::File::from_str(include_str!("cofg.yaml"), config::FileFormat::Yaml))
      .build()
      .unwrap()
      .try_deserialize::<Self>()
      .unwrap()
  }
}
