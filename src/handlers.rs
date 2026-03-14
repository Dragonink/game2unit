//! Provides [`AttachmentHandler`] implementations.

use std::{
	ffi::OsStr,
	fmt::{self, Formatter},
};

use rootcause::handlers::AttachmentHandler;

/// [`AttachmentHandler`] implementation for environment variables.
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

	fn debug(pair: &(K, V), f: &mut Formatter<'_>) -> fmt::Result {
		Self::display(pair, f)
	}
}
