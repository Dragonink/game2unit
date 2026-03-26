//! Provides a [string for inclusion in a systemd unit name](UnitNameStr).

use std::{
	borrow::Cow,
	ffi::{OsStr, OsString},
	fmt::{self, Debug, Display, Formatter},
	ops::Deref,
};

use zbus::zvariant::Str;

/// String for inclusion in a systemd unit name
///
/// **References:**
/// - [Unit Description](https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html#Description)
/// - [String Escaping for Inclusion in Unit Names](https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html#String%20Escaping%20for%20Inclusion%20in%20Unit%20Names)
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct UnitNameStr<'s>(Str<'s>);
impl<'s> UnitNameStr<'s> {
	/// Constructs a new strings for inclusion in a systemd unit name, escaping characters as needed.
	pub(crate) fn from_bytes(bytes: &'s [u8]) -> Self {
		fn push_escaped(escaped: &mut Cow<str>, b: u8) {
			use std::fmt::Write as _;

			write!(escaped.to_mut(), "\\x{b:02x}").unwrap_or_else(|_| unreachable!());
		}

		let mut escaped = Cow::Borrowed("");
		for (i, b) in bytes.iter().copied().enumerate() {
			debug_assert!(
				escaped.bytes().all(is_valid_unit_name_char),
				"escaped systemd unit name contains invalid characters"
			);
			match b {
				b @ b'.' if i == 0 => push_escaped(&mut escaped, b),
				b @ (b'/' | b'-') => push_escaped(&mut escaped, b),
				b if is_valid_unit_name_char(b) => match escaped {
					#[expect(unsafe_code, reason = "avoid UTF-8 check")]
					Cow::Borrowed(ref mut escaped) => {
						// SAFETY: systemd unit name strings are a subset of ASCII, thus valid UTF-8.
						// We just checked that the new character is valid from unit names,
						// and the previous characters have already been checked the same way.
						*escaped = unsafe { std::str::from_utf8_unchecked(&bytes[..=i]) };
					}
					Cow::Owned(ref mut escaped) => escaped.push(b.into()),
				},
				b => push_escaped(&mut escaped, b),
			}
		}
		Self(escaped.into())
	}

	/// Returns a substring which length does not exceed `limit`.
	pub(crate) fn truncate(&'s self, limit: usize) -> Self {
		if self.len() <= limit {
			self.clone()
		} else {
			const ESCAPED_LEN: usize = 4;

			let mut take = 0;
			while take < limit {
				take += match self.0.as_bytes()[take] {
					b'\\' if take + ESCAPED_LEN <= limit => ESCAPED_LEN,
					b'\\' => break,
					_ => 1,
				};
			}
			#[expect(
				clippy::string_slice,
				reason = "string is a valid systemd unit name string"
			)]
			Self(Str::from(&self.0[..take]))
		}
	}

	/// Creates an owned clone of this string.
	pub(crate) fn into_owned(self) -> UnitNameStr<'static> {
		UnitNameStr(self.0.into_owned())
	}
}
impl<'s> From<&'s str> for UnitNameStr<'s> {
	fn from(s: &'s str) -> Self {
		Self::from_bytes(s.as_bytes())
	}
}
impl From<String> for UnitNameStr<'static> {
	fn from(s: String) -> Self {
		UnitNameStr::from(s.as_str()).into_owned()
	}
}
impl<'s> From<&'s OsStr> for UnitNameStr<'s> {
	fn from(s: &'s OsStr) -> Self {
		use std::os::unix::ffi::OsStrExt as _;

		Self::from_bytes(s.as_bytes())
	}
}
impl From<OsString> for UnitNameStr<'static> {
	fn from(s: OsString) -> Self {
		UnitNameStr::from(s.as_os_str()).into_owned()
	}
}
impl Deref for UnitNameStr<'_> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}
impl Debug for UnitNameStr<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}
impl Display for UnitNameStr<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

/// Checks if the given character is a valid character for unit names.
///
/// Valid characters are:
/// - ASCII alphanumeric characters
/// - `:`
/// - `-`
/// - `_`
/// - `.`
/// - `\`
pub(super) const fn is_valid_unit_name_char(b: u8) -> bool {
	b.is_ascii_alphanumeric() || matches!(b, b':' | b'-' | b'_' | b'.' | b'\\')
}
