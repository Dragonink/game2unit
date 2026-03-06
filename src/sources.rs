//! Sources of systemd unit properties

use std::fmt::{self, Display, Formatter};

use rootcause::prelude::*;

use crate::unit_launcher::UnitLauncherArgs;

/// Source of systemd unit properties
#[derive(Debug, Clone, Copy)]
struct UnitPropertySource {
	/// Name of the source
	name: &'static str,
	/// Sources the unit properties from the environment.
	///
	/// Returns `Ok(None)` if an appropriate environment is not detected.
	source: fn() -> rootcause::Result<Option<UnitLauncherArgs>>,
}
impl Display for UnitPropertySource {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str(self.name)
	}
}

#[cfg(feature = "steam")]
mod steam;

/// Attempts to source the unit properties from all environments.
///
/// Returns `Ok(None)` if no environment was detected.
///
/// # Errors
/// Returns any error returned by a source.
pub(super) fn source_unit_properties() -> rootcause::Result<Option<UnitLauncherArgs>> {
	const SOURCES: &[UnitPropertySource] = &[
		#[cfg(feature = "steam")]
		steam::SOURCE,
	];

	for source in SOURCES {
		if let Some(args) = (source.source)()
			.context("Failed to source the systemd unit properties")
			.attach_custom::<handlers::Display, _>(source)?
		{
			return Ok(Some(args));
		}
	}
	Ok(None)
}
