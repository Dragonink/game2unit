//! Provides the [systemd unit launcher](Launcher).

use std::{
	collections::HashMap,
	ffi::{OsStr, OsString},
	fmt::Display,
	os::unix::ffi::{OsStrExt as _, OsStringExt as _},
	path::Path,
	process::Command,
};

use rootcause::{option_ext::OptionExt as _, prelude::*};

/// systemd unit launcher
///
/// The [default](Self::default()) launcher [shell command](Self::from_shell_command()) is configured at compile-time using the `GAME2UNIT_DEFAULT_LAUNCHER` environment variable.
/// If not set, it defaults to [`app2unit`](https://github.com/Vladimir-csp/app2unit).
#[derive(Debug)]
pub(super) struct Launcher(Command);
impl Launcher {
	/// Constructs a new systemd unit launcher from its path.
	///
	/// See [`Command::new()`] for more details.
	pub(super) fn new<S>(program: S) -> Self
	where
		S: AsRef<OsStr>,
	{
		Self(Command::new(program))
	}

	/// Constructs a new systemd unit launcher from a shell command.
	///
	/// # Errors
	/// Returns an error if the given shell command is invalid.
	pub(super) fn from_shell_command<S>(command: S) -> rootcause::Result<Self>
	where
		S: AsRef<OsStr>,
	{
		let mut shlex = shlex::bytes::Shlex::new(command.as_ref().as_bytes());

		let mut args = shlex.by_ref().map(OsString::from_vec);
		let mut ret = Self::new(args.next().context("Missing program path")?);
		ret.args(args);

		if shlex.had_error {
			Err(report!("Malformed shell command"))
		} else {
			Ok(ret)
		}
	}

	/// Adds an argument to pass to the launcher.
	///
	/// See [`Command::arg()`] for more details.
	///
	/// To pass multiple arguments, see [`Self::args()`].
	pub(super) fn arg<A>(&mut self, arg: A) -> &mut Self
	where
		A: AsRef<OsStr>,
	{
		self.0.arg(arg);
		self
	}

	/// Adds multiple arguments to pass to the launcher.
	///
	/// See [`Command::args()`] for more details.
	///
	/// To pass a single argument, see [`Self::arg()`].
	pub(super) fn args<A, I>(&mut self, args: I) -> &mut Self
	where
		A: AsRef<OsStr>,
		I: IntoIterator<Item = A>,
	{
		self.0.args(args);
		self
	}

	/// Adds arguments from sourced unit properties.
	pub(super) fn sourced_args(&mut self, sourced: &SourcedProperties) -> &mut Self {
		self.arg("-a").arg(&sourced.app_name);
		if let Some(game_title) = sourced.game_title.as_ref() {
			self.arg("-d").arg(game_title);
		}
		for (key, value) in &sourced.properties {
			let mut arg = key.clone();
			arg.push("=");
			arg.push(value);
			self.arg("-p").arg(arg);
		}
		if sourced.force_scope {
			self.arg("-t").arg("scope");
		}

		self
	}

	/// Executes the configured launcher, replacing this process.
	///
	/// On success this function will not return,
	/// and otherwise it will return an error.
	/// See [`std::os::unix::process::CommandExt::exec()`] for more details.
	pub(super) fn exec(&mut self) -> std::io::Error {
		const LOG_LEVEL: log::Level = log::Level::Debug;
		if log::log_enabled!(LOG_LEVEL) {
			let command = shlex::bytes::Quoter::new()
				.allow_nul(true)
				.join(
					std::iter::once(self.0.get_program())
						.chain(self.0.get_args())
						.map(OsStr::as_bytes),
				)
				.map_or_else(|_| unreachable!(), OsString::from_vec);
			log::log!(
				LOG_LEVEL,
				"Executing systemd unit launcher:\n{}",
				command.display()
			);
		}

		std::os::unix::process::CommandExt::exec(&mut self.0)
	}
}
impl Default for Launcher {
	fn default() -> Self {
		#[expect(clippy::expect_used, reason = "report bad compile-time configuration")]
		Self::from_shell_command(option_env!("GAME2UNIT_DEFAULT_LAUNCHER").unwrap_or("app2unit"))
			.expect("Default systemd unit launcher command should be valid")
	}
}

/// systemd unit properties retrieved from [sources](crate::sources)
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourcedProperties {
	/// Application name substring of the unit ID
	app_name: OsString,
	/// Game title, set as the unit description
	game_title: Option<OsString>,
	/// Additional unit properties
	properties: HashMap<OsString, OsString>,
	/// Forces the unit to be a scope.
	force_scope: bool,
}
impl SourcedProperties {
	/// Constructs a new set of systemd unit properties.
	///
	/// The application name is constructed from the given source ID and game ID.
	pub(crate) fn new<I>(source_id: &str, game_id: I) -> Self
	where
		I: Display,
	{
		Self {
			app_name: format!("{source_id}-{game_id}").into(),
			game_title: None,
			properties: HashMap::default(),
			force_scope: false,
		}
	}

	/// Sets the game title.
	pub(crate) fn game_title<S>(&mut self, game_title: S) -> &mut Self
	where
		S: AsRef<OsStr>,
	{
		self.game_title = Some(game_title.as_ref().to_os_string());
		self
	}

	/// Sets a unit property.
	///
	/// If a property with the same key is already set,
	/// its value is updated.
	fn property<K, V>(&mut self, key: K, value: V) -> &mut Self
	where
		K: AsRef<OsStr>,
		V: AsRef<OsStr>,
	{
		self.properties
			.insert(key.as_ref().to_os_string(), value.as_ref().to_os_string());
		self
	}

	/// Sets the `SourcePath` unit property.
	///
	/// See [`Self::property()`] for more details.
	pub(crate) fn source_path<P>(&mut self, source_path: P) -> &mut Self
	where
		P: AsRef<Path>,
	{
		self.property("SourcePath", source_path.as_ref())
	}

	/// Forces the unit to be a scope.
	pub(crate) const fn force_scope(&mut self) -> &mut Self {
		self.force_scope = true;
		self
	}
}
