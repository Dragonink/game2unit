//! Provides the [Steam unit property source](SOURCE).

use std::{
	env::VarError,
	fmt::{self, Debug, Display, Formatter},
	num::NonZeroU32,
	path::PathBuf,
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
/// - an App Manifest cannot be found;
/// - the found App Manifest cannot be deserialized.
fn source() -> rootcause::Result<Option<UnitLauncherArgs>> {
	let mut args = UnitLauncherArgs::new();
	args.force_scope();

	let Some(app_id) = retrieve_app_id().context("Failed to retrieve the App ID")? else {
		return Ok(None);
	};
	args.app_name(format!("steam-{app_id}"));

	let manifest_path = find_app_manifest(app_id)
		.and_then(|opt| opt.ok_or_report().map_err(Report::into_dynamic))
		.context("Could not find an App Manifest")
		.attach_custom::<AppIdHandler, _>(app_id)?;
	args.unit_source_path(&manifest_path);

	let manifest = std::fs::read_to_string(&manifest_path)
		.context("Failed to read the App Manifest")
		.attach_with(|| manifest_path.display().to_string())?;
	let manifest: AppManifest = serde_vdf::text::from_str(&manifest)
		.context("Failed to deserialize the App Manifest")
		.attach_with(|| manifest_path.display().to_string())?;
	if manifest.app_state.app_id != app_id {
		return Err(report!("Incorrect App Manifest")
			.attach(format!("App ID in Manifest: {}", manifest.app_state.app_id))
			.attach_custom::<AppIdHandler, _>(app_id));
	}
	args.game_title(manifest.app_state.name);

	Ok(Some(args))
}

/// Structure of Steam App Manifest files
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AppManifest<'a> {
	/// Metadata and state of the App
	#[serde(borrow)]
	app_state: AppState<'a>,
}

/// Metadata and state of a Steam App
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
struct AppState<'a> {
	/// App ID
	#[serde(rename = "appid")]
	app_id: AppId,
	/// App name
	name: &'a str,
}

/// Retrieves the App ID from the environment.
///
/// Returns `Ok(None)` if an App ID cannot be found in the environment.
///
/// # Errors
/// Returns an error if the parsing of the found App ID fails.
fn retrieve_app_id() -> rootcause::Result<Option<AppId>> {
	const VAR: &str = "SteamAppId";

	match std::env::var(VAR) {
		Ok(value) => value
			.parse()
			.context("Invalid App ID")
			.attach_with(|| format!("{VAR}={value:?}"))
			.map_err(Report::into_dynamic)
			.map(Some),
		Err(VarError::NotPresent) => Ok(None),
		Err(err) => Err(report!(err).into_dynamic()),
	}
}

/// Finds the Manifest of the App given its ID.
///
/// # Errors
/// Returns an error if the Steam library paths cannot be found.
fn find_app_manifest(app_id: AppId) -> rootcause::Result<Option<PathBuf>> {
	let library_paths = std::env::var_os("STEAM_COMPAT_LIBRARY_PATHS")
		.map(PathBuf::from)
		.or_else(|| dirs::data_local_dir().map(|dir| dir.join("Steam").join("steamapps")))
		.context("Could not find the library directories")?;
	Ok(std::env::split_paths(&library_paths).find_map(|library| {
		let manifest_path = library.join(format!("appmanifest_{app_id}.acf"));
		manifest_path.exists().then_some(manifest_path)
	}))
}

/// Steam App ID
#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
struct AppId(NonZeroU32);
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
