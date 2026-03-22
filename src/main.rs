#![doc = env!("CARGO_PKG_DESCRIPTION")]


use std::{ffi::OsStr, process::ExitCode};

use rootcause::{option_ext::OptionExt as _, prelude::*};

fn main() -> ExitCode {
	/// Part of the [`main()`] function that makes the program fails if an error occurs.
	///
	/// On success this function will not return,
	/// see [`CommandExt::exec()`](std::os::unix::process::CommandExt::exec()) for more details.
	///
	/// # Errors
	/// Returns an error if:
	/// - executing the command fails.
	fn critical_main() -> rootcause::Result<()> {
		let mut args = std::env::args_os().skip(1);

		exec(args).context("Failed to execute command")?;
		unreachable!();
	}

	setup_logger();

	match critical_main() {
		Ok(()) => unreachable!(),
		Err(err) => {
			log::error!("Fatal error: {err}");
			ExitCode::FAILURE
		}
	}
}

/// Executes the given command, **replacing this process**.
///
/// On success this function will not return,
/// see [`CommandExt::exec()`](std::os::unix::process::CommandExt::exec()) for more details.
///
/// # Errors
/// Returns an error if:
/// - `args` is empty;
/// - the execution of the constructed command fails.
fn exec<A, I>(args: I) -> rootcause::Result<()>
where
	A: AsRef<OsStr>,
	I: IntoIterator<Item = A>,
{
	use std::{os::unix::process::CommandExt as _, process::Command};

	let mut args = args.into_iter();
	let mut cmd = Command::new(args.next().context("Missing game executable")?);
	cmd.args(args);

	log::info!("Executing command:\n{cmd:?}");
	Err(report!(cmd.exec()).into_dynamic())
}

/// Configures and installs the application logger.
///
/// # Panics
/// Panics if a logger has previously been installed.
fn setup_logger() {
	use log::LevelFilter;

	env_logger::Builder::new()
		.format_indent(Some(8))
		.format_target(false)
		.filter_level(if cfg!(debug_assertions) {
			LevelFilter::max()
		} else {
			LevelFilter::Warn
		})
		.parse_env("GAME2UNIT_LOG")
		.init();
}
