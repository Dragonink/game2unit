//! Provides an abstraction to prepare the systemd unit launcher.

use std::{
	ffi::OsStr,
	fmt::{self, Debug, Formatter},
	os::unix::process::CommandExt as _,
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
