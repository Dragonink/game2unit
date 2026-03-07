//! Provides the [source function for a Steam App](source).

use std::path::PathBuf;

use rootcause::{option_ext::OptionExt as _, prelude::*};
use serde::Deserialize;

use crate::unit_launcher::UnitLauncherArgs;

use super::{AppId, AppIdHandler};

/// Sources the unit properties for a Steam App from the environment.
///
/// # Errors
/// Returns an error if:
/// - the App Manifest cannot be found;
/// - the found App Manifest cannot be deserialized;
/// - the found App Manifest is incorrect.
pub(super) fn source(args: &mut UnitLauncherArgs, app_id: AppId) -> rootcause::Result<()> {
	args.app_name(format!("steam-{app_id}"));

	let manifest_path = find_app_manifest(app_id)
		.and_then(|opt| opt.ok_or_report().map_err(Report::into_dynamic))
		.context("Could not find the App Manifest")
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

	Ok(())
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
