//! Source systemd unit properties for a Steam App.

use std::path::{Path, PathBuf};

use rootcause::{option_ext::OptionExt as _, prelude::*};
use serde::Deserialize;

use crate::launcher::SourcedProperties;

use super::AppId;

/// Retrieves systemd unit properties from Steam for a Steam App.
pub(super) fn source_systemd_unit_properties(app_id: AppId) -> SourcedProperties {
	log::debug!("Found App ID: {app_id}");
	let mut sourced = SourcedProperties::new(super::SOURCE_ID, app_id);

	match find_app_manifest(app_id).attach_custom::<super::AppIdHandler, _>(app_id) {
		Ok(app_manifest_path) => {
			log::debug!("Found App Manifest: {}", app_manifest_path.display());
			if let Err(err) = source_from_app_manifest(&mut sourced, &app_manifest_path)
				.attach_with(|| app_manifest_path.display().to_string())
			{
				log::debug!("Failed to source systemd unit properties from App Manifest: {err}");
			}
		}
		Err(err) => log::debug!("Could not find App Manifest: {err}"),
	}

	sourced
}

/// Finds the App Manifest given an App ID.
///
/// # Errors
/// Returns an error if:
/// - the Library paths cannot be found;
/// - the App Manifest cannot be found in the found Library paths.
fn find_app_manifest(app_id: AppId) -> rootcause::Result<PathBuf> {
	const VAR: &str = "STEAM_COMPAT_LIBRARY_PATHS";
	let library_paths = std::env::var_os(VAR)
		.map_or_else::<rootcause::Result<_>, _, _>(
			|| {
				Ok(dirs::data_local_dir()
					.context("Could not find user's local data directory")?
					.join("Steam")
					.join("steamapps")
					.into_os_string())
			},
			Ok,
		)
		.context("Could not find Library paths")?;

	std::env::split_paths(&library_paths)
		.find_map(|path| {
			let app_manifest_path = path.join(format!("appmanifest_{app_id}.acf"));
			app_manifest_path.exists().then_some(app_manifest_path)
		})
		.ok_or_report()
		.map_err(Report::into_dynamic)
}

/// Adds systemd unit properties retrieved from an App Manifest.
///
/// # Errors
/// Returns an error if:
/// - the App Manifest cannot be read;
/// - the App Manifest cannot be deserialized.
fn source_from_app_manifest<P>(sourced: &mut SourcedProperties, path: P) -> rootcause::Result<()>
where
	P: AsRef<Path>,
{
	let app_manifest = std::fs::read_to_string(&path).context("Failed to read App Manifest")?;
	let app_manifest: AppManifest =
		serde_vdf::text::from_str(&app_manifest).context("Failed to deserialize App Manifest")?;
	log::debug!("Read App Manifest:\n{app_manifest:#?}");

	sourced
		.source_path(path)
		.game_title(app_manifest.app_state.name);

	Ok(())
}

/// Structure of App Manifest files
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AppManifest<'a> {
	/// App metadata and state
	#[serde(borrow)]
	app_state: AppState<'a>,
}

/// Steam App metadata and state
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
struct AppState<'a> {
	/// Steam App ID
	#[serde(rename = "appid")]
	app_id: AppId,
	/// App name
	name: &'a str,
}
