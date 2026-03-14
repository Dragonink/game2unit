#![doc = env!("CARGO_PKG_DESCRIPTION")]

fn main() {
	setup_logger();

	match run() {
		Ok(()) => {}
		Err(err) => log::error!("Fatal error: {err}"),
	}
}

/// Main function
///
/// # Errors
/// Currently does not return any error.
#[expect(clippy::unnecessary_wraps, reason = "Result will be useful later")]
fn run() -> rootcause::Result<()> {
	log::info!("Hello, world!");

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
