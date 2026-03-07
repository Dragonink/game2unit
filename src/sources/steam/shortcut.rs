//! Provides the [source function for a Steam Shortcut](source).

use std::{collections::HashMap, env::VarError, path::PathBuf};

use rootcause::{option_ext::OptionExt as _, prelude::*};
use serde::Deserialize;

use crate::unit_launcher::UnitLauncherArgs;

use super::{AppId, AppIdHandler};

/// Sources the unit properties for a Steam Shortcut from the environment.
///
/// # Errors
/// Returns an error if:
/// - the Shortcuts file cannot be found;
/// - the found Shortcuts file cannot be deserialized;
/// - the Shortcut cannot be found.
pub(super) fn source(args: &mut UnitLauncherArgs, app_id: AppId) -> rootcause::Result<()> {
	args.app_name(format!("steam-{app_id}"));

	let shortcuts_path = find_shortcuts()
		.and_then(|opt| opt.ok_or_report().map_err(Report::into_dynamic))
		.context("Could not find the Shortcuts file")?;
	args.unit_source_path(&shortcuts_path);

	let shortcuts = std::fs::read(&shortcuts_path)
		.context("Failed to read the Shortcuts file")
		.attach_with(|| shortcuts_path.display().to_string())?;
	let shortcuts: Shortcuts = serde_vdf::binary::from_bytes(&shortcuts)
		.context("Failed to deserialize the Shortcuts file")
		.attach_with(|| shortcuts_path.display().to_string())?;
	let shortcut = shortcuts
		.shortcuts
		.iter()
		.find(|shortcut| shortcut.app_id == app_id)
		.context("Could not find the Shortcut")
		.attach_custom::<AppIdHandler, _>(app_id)
		.attach_with(|| shortcuts_path.display().to_string())?;
	args.game_title(shortcut.app_name);

	Ok(())
}

/// Structure of Steam Shortcuts files
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
struct Shortcuts<'a> {
	/// List of Shortcuts
	#[serde(borrow)]
	shortcuts: Vec<Shortcut<'a>>,
}

/// Metadata of a Shortcut
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Shortcut<'a> {
	/// App ID
	#[serde(rename = "appid")]
	app_id: AppId,
	/// App name
	app_name: &'a str,
}

/// Finds the Shortcuts file of the User playing.
///
/// # Errors
/// Returns an error if:
/// - the XDG Data Directory cannot be found;
/// - the Login Users file cannot be deserialized.
fn find_shortcuts() -> rootcause::Result<Option<PathBuf>> {
	let steam_dir = dirs::data_local_dir()
		.context("Could not find the XDG Data Directory")?
		.join("Steam");

	let app_user = match std::env::var("SteamAppUser") {
		Ok(value) => value,
		Err(VarError::NotPresent) => return Ok(None),
		Err(err) => {
			return Err(report!(err)
				.context("Failed to retrieve the User")
				.into_dynamic());
		}
	};

	let users_path = steam_dir.join("config").join("loginusers.vdf");
	let users = std::fs::read_to_string(&users_path)
		.context("Failed to read the Login Users file")
		.attach_with(|| users_path.display().to_string())?;
	let users: LoginUsers = serde_vdf::text::from_str(&users)
		.context("Failed to deserialize the Login Users file")
		.attach_with(|| users_path.display().to_string())?;

	Ok(users
		.users
		.iter()
		.find_map(|(id, user)| (user.account_name == app_user).then_some(id))
		.map(|id| {
			steam_dir
				.join("userdata")
				.join(format!("{}", id.to_id32()))
				.join("config")
				.join("shortcuts.vdf")
		}))
}

/// Structure of Steam Login Users files
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct LoginUsers<'a> {
	/// Map of Login User metadata
	#[serde(borrow)]
	users: HashMap<UserId, LoginUser<'a>>,
}

/// Steam User ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
struct UserId(u64);
impl UserId {
	/// Converts this 64-bit User ID into a 32-bit User ID.
	#[expect(
		clippy::cast_possible_truncation,
		reason = "taking the least significant u32"
	)]
	const fn to_id32(self) -> u32 {
		self.0 as u32
	}
}

/// Metadata of a Steam Login User
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LoginUser<'a> {
	/// Account name
	account_name: &'a str,
}
