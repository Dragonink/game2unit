//! Provides the [Steam unit property source](SOURCE).

mod shortcut;
mod steam_app;

use std::{
	env::VarError,
	fmt::{self, Debug, Display, Formatter},
	num::NonZeroU32,
	str::FromStr,
};

use rootcause::{handlers::AttachmentHandler, option_ext::OptionExt as _, prelude::*};
use serde::Deserialize;

use crate::unit_launcher::UnitLauncherArgs;

use super::UnitPropertySource;

/// Steam unit property source
pub(super) const SOURCE: UnitPropertySource = UnitPropertySource {
	name: "Steam",
	source,
};

/// Sources the unit properties from a Steam environment.
///
/// Returns `Ok(None)` if a Steam environment is not detected.
///
/// # Errors
/// Returns an error if:
/// - the parsing of the found App ID fails;
/// - [`steam_app::source()`] fails;
/// - [`shortcut::source()`] fails.
fn source() -> rootcause::Result<Option<UnitLauncherArgs>> {
	let mut args = UnitLauncherArgs::new();
	args.force_scope();

	match retrieve_game_id().context("Failed to retrieve the Game ID")? {
		Some(GameId::SteamApp(app_id)) => steam_app::source(&mut args, app_id)?,
		Some(GameId::Shortcut(app_id)) => shortcut::source(&mut args, app_id)?,
		None => return Ok(None),
	}

	Ok(Some(args))
}

/// Steam Game ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameId {
	/// Steam App ID
	SteamApp(AppId),
	/// Shortcut App ID
	Shortcut(AppId),
}
impl GameId {
	/// Creates a Game ID if the given value is valid.
	fn new(value: u64) -> Option<Self> {
		u32::try_from(value).map_or_else(
			|_| AppId::new((value >> u32::BITS) as _).map(Self::Shortcut),
			|value| AppId::new(value).map(Self::SteamApp),
		)
	}
}

/// Retrieves the Game ID from the environment.
///
/// # Errors
/// Returns an error if the parsing of the found Game ID fails.
fn retrieve_game_id() -> rootcause::Result<Option<GameId>> {
	const VAR: &str = "SteamGameId";

	match std::env::var(VAR) {
		Ok(value) => value
			.parse()
			.map_err(From::from)
			.and_then(|value| {
				GameId::new(value)
					.ok_or_report()
					.map_err(Report::into_dynamic)
			})
			.context("Invalid Game ID")
			.attach_with(|| format!("{VAR}={value:?}"))
			.map_err(Report::into_dynamic)
			.map(Some),
		Err(VarError::NotPresent) => Ok(None),
		Err(err) => Err(report!(err).into_dynamic()),
	}
}

/// Steam App ID
#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
struct AppId(NonZeroU32);
impl AppId {
	/// Creates an App ID if the given value is valid.
	fn new(value: u32) -> Option<Self> {
		NonZeroU32::new(value).map(Self)
	}
}
impl FromStr for AppId {
	type Err = <NonZeroU32 as FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.parse().map(Self)
	}
}
impl Debug for AppId {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self.0, f)
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
	fn debug(value: &AppId, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "AppId({value})")
	}

	fn display(value: &AppId, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "App ID: {value}")
	}
}
