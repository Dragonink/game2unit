#![doc = env!("CARGO_PKG_DESCRIPTION")]

use std::process::ExitCode;

/// Main function
///
/// # Errors
/// Returns an error if:
fn run() -> rootcause::Result<()> {
	Ok(())
}

#[expect(clippy::print_stderr, reason = "display error report")]
fn main() -> ExitCode {
	match run() {
		Ok(()) => ExitCode::SUCCESS,
		Err(err) => {
			eprintln!("{err}");
			ExitCode::FAILURE
		}
	}
}
