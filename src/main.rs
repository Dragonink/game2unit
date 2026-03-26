#![doc = env!("CARGO_PKG_DESCRIPTION")]

mod env;
mod systemd;

use std::{ffi::OsStr, path::Path, process::ExitCode};

use rootcause::{option_ext::OptionExt as _, prelude::*};

use self::systemd::{UnitName, UnitNameStr, UnitProperties, UnitType};

fn main() -> ExitCode {
	/// Part of the [`main()`] function that makes the program fails if an error occurs.
	///
	/// On success this function will not return,
	/// see [`CommandExt::exec()`](std::os::unix::process::CommandExt::exec()) for more details.
	///
	/// # Errors
	/// Returns an error if:
	/// - starting the systemd unit fails;
	/// - executing the command fails.
	fn critical_main(
		launcher: Option<UnitNameStr>,
		app_name: Option<UnitNameStr>,
		unit_properties: UnitProperties,
		unit_type: UnitType,
	) -> rootcause::Result<()> {
		let mut args = std::env::args_os().skip(1).peekable();

		let app_name: UnitNameStr = match app_name {
			Some(app_name) => app_name,
			None => args
				.peek()
				.and_then(|s| Path::new(s).file_name())
				.context("Missing game executable")?
				.into(),
		};
		start_unit(launcher, app_name, unit_properties, unit_type)
			.context("Failed to start systemd unit")?;

		exec(args).context("Failed to execute command")?;
		unreachable!();
	}

	setup_logger();

	let launcher = None;
	let app_name = None;
	let unit_properties = UnitProperties::with_capacity(5);

	match critical_main(launcher, app_name, unit_properties, UnitType::Scope) {
		Ok(()) => unreachable!(),
		Err(err) => {
			log::error!("Fatal error: {err}");
			ExitCode::FAILURE
		}
	}
}

/// Creates and starts a systemd unit.
///
/// # Errors
/// Returns an error if:
/// - the configuration read from the environment is invalid;
/// - starting the systemd unit fails.
fn start_unit(
	launcher: Option<UnitNameStr>,
	app_name: UnitNameStr,
	mut properties: UnitProperties,
	unit_type: UnitType,
) -> rootcause::Result<()> {
	use self::systemd::{DEFAULT_SLICE, GRAPHICAL_SESSION, UnitCollectMode};
	use rustix::process::PidfdFlags;

	let launcher = launcher.or_else(|| match env::current_desktop() {
		Ok(desktop) => {
			log::trace!("Retrieved current desktop: {desktop}");
			Some(desktop)
		}
		Err(err) => {
			log::warn!("Could not retrieve current desktop: {err}");
			None
		}
	});
	let unit_name = UnitName::new_app(launcher, app_name, unit_type);
	log::trace!("Constructed systemd unit name: {unit_name}");

	properties.slice(
		env::config_slice()
			.context("Configured systemd slice name is invalid")?
			.unwrap_or(DEFAULT_SLICE),
	);
	properties.add_after(GRAPHICAL_SESSION);
	properties.add_part_of(GRAPHICAL_SESSION);
	properties.collect_mode(UnitCollectMode::InactiveOrFailed);
	properties.add_pidfd({
		// NOTE: We open a PIDFD of our own PID and give it to the systemd unit, since we will replace this process with the command.
		// REF: systemd-run <https://github.com/systemd/systemd/blob/53d5f5c02f74105b2205c5181eba98cb4c5568d4/src/run/run.c#L1687>
		// REF: runapp <https://github.com/c4rlo/runapp/blob/4c0a85fbca9b23cef7b9e0d2540ce6f6b151a28f/src/main.cpp#L179>
		let pid = rustix::process::getpid();
		rustix::process::pidfd_open(pid, PidfdFlags::empty())
			.context("Failed to open PIDFD")
			.attach(pid)?
	});

	systemd::start_unit(&unit_name, &properties)
		.context("Failed to start systemd unit")
		.attach_with(|| unit_name.clone())
		.attach_with(|| properties.to_string())?;
	log::info!("Started systemd unit {unit_name}");

	Ok(())
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
