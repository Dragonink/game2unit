//! Provides [systemd unit property sources](Source).

use std::fmt::{self, Display, Formatter};

use crate::launcher::SourcedProperties;

/// Attemps to retrieve systemd unit properties from all sources.
pub(super) fn source_systemd_unit_properties() -> Option<SourcedProperties> {
	const SOURCES: &[Source] = &[
	];
	SOURCES.iter().find_map(|source| {
		log::debug!("Attempting to source systemd unit properties from {source}...");
		match (source.source_systemd_unit_properties)() {
			Ok(sourced) => {
				log::info!("Sourced systemd unit properties from {source}");
				Some(sourced)
			}
			Err(err) => {
				log::debug!("Failed to source systemd unit properties from {source}: {err}");
				None
			}
		}
	})
}

/// systemd unit property source
#[derive(Debug, Clone, Copy)]
struct Source {
	/// Source name
	name: &'static str,
	/// Retrieves systemd unit properties from this source.
	source_systemd_unit_properties: fn() -> rootcause::Result<SourcedProperties>,
}
impl Display for Source {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str(self.name)
	}
}
