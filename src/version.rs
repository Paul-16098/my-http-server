#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[derive(serde::Serialize)]
pub(crate) struct Version {
	pub(crate) version: &'static str,
	profile: &'static str,
	commit_hash: &'static str,
	env_suffix: &'static str,
	features: &'static str,
}
pub(crate) const VERSION: Version = Version::new();

impl std::fmt::Display for Version {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}({} Profile)-{}({})[f:{}]",
			self.version,
			self.profile,
			self.commit_hash,
			self.env_suffix,
			if self.features.is_empty() {
				"none"
			} else {
				self.features
			},
		)
	}
}

impl Version {
	pub const fn new() -> Self {
		Self {
			version: env!("CARGO_PKG_VERSION"),
			profile: env!("PROFILE"),
			commit_hash: env!("commit_hash"),
			env_suffix: env!("env_suffix"),
			features: env!("FEATURES"),
		}
	}
}

impl Default for Version {
	fn default() -> Self {
		Self::new()
	}
}
