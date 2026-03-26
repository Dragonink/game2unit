//! Provides functions to fetch data from the environment.

use std::{
	env::VarError,
	ffi::{OsStr, OsString},
	fmt::{self, Formatter},
};

use rootcause::{handlers::AttachmentHandler, prelude::*, report_collection::ReportCollection};

use crate::systemd::{UnitName, UnitNameStr};

/// Fetches the environment variable `key` from the current process.
///
/// See [`std::env::var_os()`] for more details.
///
/// # Errors
/// Returns an error if:
/// - the variable is not set;
/// - there is another error.
pub(crate) fn var_os<K>(key: K) -> rootcause::Result<OsString, VarError>
where
	K: AsRef<OsStr>,
{
	std::env::var_os(&key).map_or_else(
		|| Err(report!(VarError::NotPresent).attach(key.as_ref().display().to_string())),
		Ok,
	)
}

/// Fetches the environment variable `key` from the current process.
///
/// See [`std::env::var()`] for more details.
///
/// # Errors
/// Returns an error if:
/// - the variable is not set;
/// - the variable is not valid Unicode;
/// - there is another error.
pub(crate) fn var<K>(key: K) -> rootcause::Result<String, VarError>
where
	K: AsRef<OsStr>,
{
	var_os(&key).and_then(|var| {
		var.into_string().map_err(|var| {
			report!(VarError::NotUnicode(var.clone()))
				.attach_custom::<EnvVarHandler, _>((key.as_ref().to_os_string(), var))
		})
	})
}

/// Fetches the environment variable `key` from the current process,
/// returning an error if it is empty.
///
/// See [`std::env::var()`] for more details.
///
/// # Errors
/// Returns an error if:
/// - the variable is not set;
/// - the variable is empty;
/// - the variable is not valid Unicode;
/// - there is another error.
pub(crate) fn var_nonempty<K>(key: K) -> rootcause::Result<String>
where
	K: AsRef<OsStr>,
{
	let var = var(&key)?;
	if var.is_empty() {
		Err(report!("Environment variable is empty").attach(key.as_ref().display().to_string()))
	} else {
		Ok(var)
	}
}

/// Retrieves the current desktop from the environment.
///
/// Retrieving is attempted from the following environment variables, in that order:
/// - [`XDG_SESSION_DESKTOP`](https://www.freedesktop.org/software/systemd/man/latest/pam_systemd.html#desktop=)
/// - [`XDG_CURRENT_DESKTOP`](https://specifications.freedesktop.org/desktop-entry/latest/recognized-keys.html)
///
/// # Errors
/// Returns an error if the current desktop cannot be retrieved.
pub(super) fn current_desktop() -> rootcause::Result<UnitNameStr<'static>> {
	const SESSION_VAR: &str = "XDG_SESSION_DESKTOP";
	const CURRENT_VAR: &str = "XDG_CURRENT_DESKTOP";

	let mut reports = ReportCollection::new();
	reports.push(match var_nonempty(SESSION_VAR) {
		Ok(val) => return Ok(val.into()),
		Err(err) => err.into_cloneable(),
	});
	reports.push(match var_nonempty(CURRENT_VAR) {
		Ok(var) => {
			return Ok(var
				.split(':')
				.next()
				.unwrap_or_else(|| unreachable!())
				.to_owned()
				.into());
		}
		Err(err) => err.into_cloneable(),
	});
	Err(reports
		.context("Failed to retrieve the current dekstop")
		.into_dynamic())
}

/// Retrieves the configured systemd slice to put the created units in.
///
/// Attempts to read the slice name from the `GAME2UNIT_SLICE` environment variable.
/// Returns `Ok(None)` if the variable is not set.
///
/// # Errors
/// Returns an error if the the slice name is invalid.
pub(super) fn config_slice() -> rootcause::Result<Option<UnitName<'static>>> {
	const VAR: &str = "GAME2UNIT_SLICE";

	match std::env::var(VAR) {
		Ok(var) => var
			.clone()
			.try_into()
			.map(Some)
			.attach_custom::<EnvVarHandler, _>((VAR, var)),
		Err(VarError::NotPresent) => Ok(None),
		Err(VarError::NotUnicode(var)) => Err(report!(VarError::NotUnicode(var.clone()))
			.attach_custom::<EnvVarHandler, _>((VAR, var))
			.into_dynamic()),
	}
}

/// [`AttachmentHandler`] implementation for environment variables
pub(crate) struct EnvVarHandler;
impl<K, V> AttachmentHandler<(K, V)> for EnvVarHandler
where
	K: AsRef<OsStr>,
	V: AsRef<OsStr>,
{
	fn display(&(ref key, ref value): &(K, V), f: &mut Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}={:?}",
			key.as_ref().display(),
			value.as_ref().display()
		)
	}

	fn debug(value: &(K, V), f: &mut Formatter<'_>) -> fmt::Result {
		Self::display(value, f)
	}
}
