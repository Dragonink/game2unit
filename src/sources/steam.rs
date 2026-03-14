//! Source systemd unit properties from Steam.

mod steam_app;

use std::{
	fmt::{self, Display, Formatter},
	num::NonZeroU32,
	str::FromStr,
};

use rootcause::{handlers::AttachmentHandler, prelude::*};
use serde::Deserialize;

use crate::launcher::SourcedProperties;

use super::Source;

/// Steam systemd unit property source
pub(super) const SOURCE: Source = Source {
	name: "Steam",
	source_systemd_unit_properties,
};

/// Source ID
const SOURCE_ID: &str = "steam";

/// Retrieves systemd unit properties from Steam.
///
/// # Errors
/// Returns an error if a Steam environment cannot be found.
fn source_systemd_unit_properties() -> rootcause::Result<SourcedProperties> {
	const APP_ID_VAR: &str = "SteamAppId";
	let app_id = std::env::var(APP_ID_VAR).attach(APP_ID_VAR)?;
	app_id
		.parse()
		.map(AppId::new)
		.context("Invalid App ID")
		.attach_custom::<crate::handlers::EnvVarHandler, _>((APP_ID_VAR, app_id.clone()))?
		.map_or_else(
			|| {
				Err(report!("Invalid App ID")
					.attach_custom::<crate::handlers::EnvVarHandler, _>((APP_ID_VAR, app_id)))
			},
			|app_id| Ok(steam_app::source_systemd_unit_properties(app_id)),
		)
		.map(|mut sourced| {
			sourced.force_scope();
			sourced
		})
}

/// Steam App ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
struct AppId(NonZeroU32);
impl AppId {
	/// Constructs a new `AppId`.
	///
	/// Returns `None` if the given value is invalid.
	fn new(value: u32) -> Option<Self> {
		NonZeroU32::new(value).map(Self)
	}
}
impl TryFrom<u32> for AppId {
	type Error = <NonZeroU32 as TryFrom<u32>>::Error;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		NonZeroU32::try_from(value).map(Self)
	}
}
impl FromStr for AppId {
	type Err = <NonZeroU32 as FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.parse().map(Self)
	}
}
impl Display for AppId {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

/// [`AttachmentHandler`] implementation for [`AppId`]
struct AppIdHandler;
impl AttachmentHandler<AppId> for AppIdHandler {
	fn display(value: &AppId, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "App ID: {value}")
	}

	fn debug(value: &AppId, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "AppId({value:?})")
	}
}
