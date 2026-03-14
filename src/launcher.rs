//! Provides the [systemd unit launcher](Launcher).

use std::{
	ffi::{OsStr, OsString},
	os::unix::ffi::{OsStrExt as _, OsStringExt as _},
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

	/// Adds multiple arguments to pass to the launcher.
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
}
impl Default for Launcher {
	fn default() -> Self {
		#[expect(clippy::expect_used, reason = "report bad compile-time configuration")]
		Self::from_shell_command(option_env!("GAME2UNIT_DEFAULT_LAUNCHER").unwrap_or("app2unit"))
			.expect("Default systemd unit launcher command should be valid")
	}
}
