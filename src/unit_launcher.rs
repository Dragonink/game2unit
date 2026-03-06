//! Provides an abstraction to prepare the systemd unit launcher.
//!
//! # Compatible systemd unit launchers
//! A systemd unit launching program is compatible if it accepts the following options:
//!
//! Option | Description | [`UnitLauncherArgs`] method
//! :-:|:-:|:-:
//! **`-t`** | Type of unit | [`force_scope`](UnitLauncherArgs::force_scope)
//! **`-a`** | App name part of the unit ID | [`app_name`](UnitLauncherArgs::app_name)
//! **`-d`** | Unit description | [`game_title`](UnitLauncherArgs::game_title)
//! **`-p`** | Additional unit property | [`unit_property`](UnitLauncherArgs::unit_property)

use std::{
	ffi::{OsStr, OsString},
	fmt::{self, Debug, Formatter},
	os::unix::process::CommandExt as _,
	path::Path,
	process::Command,
};

use rootcause::prelude::*;

/// systemd unit launcher
///
/// The [default](Self::default()) unit launcher is [`app2unit`](https://github.com/Vladimir-csp/app2unit).
/// That can be changed at compile-time by setting the environment variable `GAME2UNIT_DEFAULT_UNIT_LAUNCHER`.
pub(super) struct UnitLauncher(Command);
impl UnitLauncher {
	/// Constructs a new systemd unit launcher.
	///
	/// See [`Command::new()`] for more details.
	pub(super) fn new<S>(program: S) -> Self
	where
		S: AsRef<OsStr>,
	{
		Self(Command::new(program))
	}

	/// Adds multiple arguments to the unit launcher.
	///
	/// See [`Command::args()`] for more details.
	pub(super) fn args<A, I>(&mut self, args: I) -> &mut Self
	where
		A: AsRef<OsStr>,
		I: IntoIterator<Item = A>,
	{
		self.0.args(args);
		self
	}

	/// Executes the unit launcher, replacing this process.
	///
	/// On success this function will not return
	/// (so its effective signature should be `Result<!, Report>` but [`!`] is a nightly feature :( ).
	/// See [`std::os::unix::process::CommandExt::exec()`] for more details.
	///
	/// # Errors
	/// Returns an error if executing the unit launcher fails.
	pub(super) fn exec(&mut self) -> rootcause::Result<(), std::io::Error> {
		Err(report!(self.0.exec()))
	}
}
impl Default for UnitLauncher {
	fn default() -> Self {
		Self::new(option_env!("GAME2UNIT_DEFAULT_UNIT_LAUNCHER").unwrap_or("app2unit"))
	}
}
impl Debug for UnitLauncher {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

/// Collection of arguments to be added to a [`UnitLauncher`]
#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub(crate) struct UnitLauncherArgs(Vec<OsString>);
impl UnitLauncherArgs {
	/// Constructs a new, empty argument collection.
	///
	/// The collection will not allocate until arguments are pushed into it.
	pub(crate) const fn new() -> Self {
		Self(Vec::new())
	}

	/// Pushes the given argument into the collection.
	pub(crate) fn arg<A>(&mut self, arg: A) -> &mut Self
	where
		A: AsRef<OsStr>,
	{
		self.0.push(arg.as_ref().to_os_string());
		self
	}

	/// Forces the unit to be a scope.
	///
	/// This is the **`-t` option** of the unit launcher.
	pub(crate) fn force_scope(&mut self) -> &mut Self {
		self.arg("-t").arg("scope")
	}

	/// Sets the app name part of the unit ID.
	///
	/// This is the **`-a` option** of the unit launcher.
	pub(crate) fn app_name<S>(&mut self, name: S) -> &mut Self
	where
		S: AsRef<OsStr>,
	{
		self.arg("-a").arg(name)
	}

	/// Sets the title of the game (the unit description).
	///
	/// This is the **`-d` option** of the unit launcher.
	pub(crate) fn game_title<S>(&mut self, title: S) -> &mut Self
	where
		S: AsRef<OsStr>,
	{
		self.arg("-d").arg(title)
	}

	/// Sets an additional unit property.
	///
	/// This is the **`-p` option** of the unit launcher.
	///
	/// # Predefined properties
	/// Unit property | Method
	/// :-:|:-:
	/// **`SourcePath`** | [`unit_source_path`](Self::unit_source_path)
	pub(crate) fn unit_property<K, V>(&mut self, key: K, value: V) -> &mut Self
	where
		K: AsRef<OsStr>,
		V: AsRef<OsStr>,
	{
		let mut property = key.as_ref().to_os_string();
		property.push("=");
		property.push(value);

		self.arg("-p").arg(property)
	}

	/// Sets the [`SourcePath`](https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html#SourcePath=) unit property.
	pub(crate) fn unit_source_path<P>(&mut self, path: P) -> &mut Self
	where
		P: AsRef<Path>,
	{
		self.unit_property("SourcePath", path.as_ref())
	}
}
impl Debug for UnitLauncherArgs {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}
impl IntoIterator for UnitLauncherArgs {
	type Item = OsString;
	type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}
