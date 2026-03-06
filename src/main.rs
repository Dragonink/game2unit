#![doc = env!("CARGO_PKG_DESCRIPTION")]

use std::process::ExitCode;

use rootcause::prelude::*;

use self::unit_launcher::UnitLauncher;

mod unit_launcher;

/// Main function
///
/// On success this function will not return
/// (so its effective signature should be `Result<!, Report>` but [`!`] is a nightly feature :( ).
/// See [`std::os::unix::process::CommandExt::exec()`] for more details.
///
/// # Errors
/// Returns an error if:
/// - executing the unit launcher returns an error.
fn run() -> rootcause::Result<()> {
	let mut unit_launcher = std::env::var_os("GAME2UNIT_UNIT_LAUNCHER")
		.map(UnitLauncher::new)
		.unwrap_or_default();

	unit_launcher
		.args(std::env::args_os().skip(1))
		.exec()
		.context("Failed to execute the systemd unit launcher")
		.map_err(Report::into_dynamic)
}

#[expect(clippy::print_stderr, reason = "display error report")]
fn main() -> ExitCode {
	match run() {
		Ok(()) => unreachable!(),
		Err(err) => {
			eprintln!("{err}");
			ExitCode::FAILURE
		}
	}
}
