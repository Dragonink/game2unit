#![doc = env!("CARGO_PKG_DESCRIPTION")]

fn main() {
	setup_logger();

	log::info!("Hello, world!");
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
