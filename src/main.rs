#![doc = env!("CARGO_PKG_DESCRIPTION")]

use rootcause::prelude::*;

use self::launcher::Launcher;

mod handlers;
mod launcher;

fn main() {
	setup_logger();

	match launch_systemd_unit() {
		Ok(()) => {}
		Err(err) => log::error!("Fatal error: {err}"),
	}
}

/// Launches the command passed as this program's arguments through a [systemd unit launcher](Launcher).
///
/// # Errors
/// Returns an error if:
/// - the systemd unit launcher cannot be constructed.
fn launch_systemd_unit() -> rootcause::Result<()> {
	const LAUNCHER_VAR: &str = "GAME2UNIT_LAUNCHER";
	let _launcher = match std::env::var_os(LAUNCHER_VAR) {
		Some(command) => Launcher::from_shell_command(&command)
			.context("Failed to construct systemd unit launcher")
			.attach_custom::<handlers::EnvVarHandler, _>((LAUNCHER_VAR, command))?,
		None => Launcher::default(),
	};

	Ok(())
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
			LevelFilter::Debug
		} else {
			LevelFilter::Warn
		})
		.parse_env("GAME2UNIT_LOG")
		.init();
}
