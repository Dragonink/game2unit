//! Source systemd unit properties for a Shortcut.

use std::{
	collections::HashMap,
	fmt::{self, Display, Formatter},
	num::NonZeroU64,
	path::{Path, PathBuf},
	str::FromStr,
};

use rootcause::{option_ext::OptionExt as _, prelude::*};
use serde::Deserialize;

use crate::launcher::SourcedProperties;

use super::AppId;

/// Retrieves systemd unit properties from Steam for a Shortcut.
///
/// # Errors
/// Returns an error if the App ID cannot be retrived.
pub(super) fn source_systemd_unit_properties() -> rootcause::Result<SourcedProperties> {
	let app_id = retrieve_app_id().context("Failed to retrieve App ID")?;
	let mut sourced = SourcedProperties::new(super::SOURCE_ID, app_id);

	match find_shortcuts_file() {
		Ok(shortcuts_path) => {
			log::debug!("Found Shortcuts: {}", shortcuts_path.display());
			if let Err(err) = source_from_shortcuts(&mut sourced, &shortcuts_path, app_id)
				.attach_with(|| shortcuts_path.display().to_string())
			{
				log::debug!("Failed to source systemd unit properties from Shortcuts: {err}");
			}
		}
		Err(err) => log::debug!("Could not find Shortcuts: {err}"),
	}

	Ok(sourced)
}

/// Retrieves the App ID.
///
/// # Errors
/// Returns an error if the App ID cannot be retrieved.
fn retrieve_app_id() -> rootcause::Result<AppId> {
	const GAME_ID_VAR: &str = "SteamGameId";
	let game_id = std::env::var(GAME_ID_VAR).attach(GAME_ID_VAR)?;
	let game_id: GameId =
		game_id
			.parse()
			.context("Invalid Game ID")
			.attach_custom::<crate::handlers::EnvVarHandler, _>((GAME_ID_VAR, game_id))?;
	log::debug!("Found Game ID: {game_id}");

	let app_id = AppId::try_from(game_id)
		.context("Failed to convert Game ID to App ID")
		.attach(game_id)?;
	log::debug!("Coverted to App ID: {app_id}");

	Ok(app_id)
}

/// Steam Game ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GameId(NonZeroU64);
impl FromStr for GameId {
	type Err = <NonZeroU64 as FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.parse().map(Self)
	}
}
impl TryFrom<GameId> for AppId {
	type Error = <Self as TryFrom<u32>>::Error;

	fn try_from(value: GameId) -> Result<Self, Self::Error> {
		Self::try_from((value.0.get() >> u32::BITS) as u32)
	}
}
impl Display for GameId {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

/// Finds the Shortcuts file.
///
/// # Errors
/// Returns an error if:
/// - the Steam directory cannot be found;
/// - the App User cannot be found;
/// - the User ID of the App User cannot be found.
fn find_shortcuts_file() -> rootcause::Result<PathBuf> {
	const APP_USER_VAR: &str = "SteamAppUser";

	let steam_dir = dirs::data_local_dir()
		.context("Could not find user's local data directory")?
		.join("Steam");

	let app_user = std::env::var(APP_USER_VAR).attach(APP_USER_VAR)?;
	log::debug!("Found App User: {app_user}");

	let login_users_path = steam_dir.join("config").join("loginusers.vdf");
	let user_id = retrieve_user_id(&login_users_path, &app_user)
		.attach_with(|| login_users_path.display().to_string())?;
	log::debug!("Retrieved User ID: {user_id}");

	Ok(steam_dir
		.join("userdata")
		.join(user_id.to_id32().to_string())
		.join("config")
		.join("shortcuts.vdf"))
}

/// Steam User ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
struct UserId(NonZeroU64);
impl UserId {
	/// Converts this User ID to its 32-bit version.
	#[expect(
		clippy::cast_possible_truncation,
		reason = "taking the least significant u32"
	)]
	const fn to_id32(self) -> u32 {
		self.0.get() as _
	}
}
impl Display for UserId {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

/// Retrieves the User ID given an App User by searching the Login Users.
///
/// # Errors
/// Returns an error if:
/// - the Login Users cannot be read;
/// - the given App User cannot be found among the Login Users.
fn retrieve_user_id<P>(login_users_path: P, app_user: &str) -> rootcause::Result<UserId>
where
	P: AsRef<Path>,
{
	let login_users =
		std::fs::read_to_string(login_users_path).context("Failed to read Login Users")?;
	let login_users: LoginUsers =
		serde_vdf::text::from_str(&login_users).context("Failed to deserialize Login Users")?;
	login_users
		.users
		.into_iter()
		.find_map(|(user_id, user)| (user.account_name == app_user).then_some(user_id))
		.ok_or_report()
		.context("Could not find User ID")
		.attach_with(|| format!("App User: {app_user}"))
		.map_err(Report::into_dynamic)
}

/// Structure of Login Users file
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct LoginUsers<'a> {
	/// Map of Steam Login User metadata
	#[serde(borrow)]
	users: HashMap<UserId, LoginUser<'a>>,
}

/// Steam Login User metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LoginUser<'a> {
	/// Account name
	account_name: &'a str,
}

/// Adds systemd unit properties retrieved from a Shortcut.
///
/// # Errors
/// Returns an error if:
/// - the Shortcuts cannot be read;
/// - the Shortcuts cannot be deserialized;
/// - the App ID cannot be found among Shortcuts.
fn source_from_shortcuts<P>(
	sourced: &mut SourcedProperties,
	path: P,
	app_id: AppId,
) -> rootcause::Result<()>
where
	P: AsRef<Path>,
{
	let shortcuts = std::fs::read(&path).context("Failed to read Shortcuts")?;
	let shortcuts: Shortcuts =
		serde_vdf::binary::from_bytes(&shortcuts).context("Failed to deserialize Shortcuts")?;
	let shortcut = shortcuts
		.shortcuts
		.iter()
		.find(|shortcut| shortcut.app_id == app_id)
		.ok_or_report()
		.context("Could not find Shortcut")
		.attach_custom::<super::AppIdHandler, _>(app_id)?;
	log::debug!("Read Shortcut:\n{shortcut:#?}");

	sourced.source_path(path);
	if let Some(game_title) = shortcut.app_name {
		sourced.game_title(game_title);
	}

	Ok(())
}

/// Structure of Shortcuts file
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
struct Shortcuts<'a> {
	/// List of Shortcuts
	#[serde(borrow)]
	shortcuts: Vec<Shortcut<'a>>,
}

/// Shortcut metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Shortcut<'a> {
	/// Steam App ID
	#[serde(rename = "appid")]
	app_id: AppId,
	/// App name
	#[serde(with = "serde_vdf::binary::optional")]
	app_name: Option<&'a str>,
}
