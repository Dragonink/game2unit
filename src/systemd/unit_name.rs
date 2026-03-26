//! Provides [`UnitName`] and associated types.

mod str;

use std::fmt::{self, Debug, Display, Formatter};

use rootcause::prelude::*;
use serde::{Deserialize, Serialize};
use zbus::zvariant::{OwnedValue, Str, Type, Value};

pub(crate) use self::str::UnitNameStr;

/// Default slice to put created units in
pub(crate) const DEFAULT_SLICE: UnitName = UnitName(Str::from_static("app.slice"));
/// [`graphical-session.target`](https://www.freedesktop.org/software/systemd/man/latest/systemd.special.html#graphical-session.target)
pub(crate) const GRAPHICAL_SESSION: UnitName =
	UnitName(Str::from_static("graphical-session.target"));

/// Name of a systemd unit
#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Type, Value, OwnedValue)]
#[serde(
	expecting = "systemd unit name",
	try_from = "Str<'s>",
	into = "Str<'s>"
)]
pub(crate) struct UnitName<'s>(#[serde(borrow)] Str<'s>);
impl UnitName<'_> {
	/// Maximum length of a unit name
	pub(crate) const MAX_LEN: usize = 255;

	/// Constructs the name of the systemd unit for an app.
	pub(crate) fn new_app<'l, 'a, L, A>(
		launcher: Option<L>,
		app_name: A,
		unit_type: UnitType,
	) -> Self
	where
		L: Into<UnitNameStr<'l>>,
		A: Into<UnitNameStr<'a>>,
	{
		use rand::RngExt as _;
		use std::fmt::Write as _;

		const PREFIX: &str = "app";
		const SEPARATOR: &str = "-";

		let mut ret = String::with_capacity(Self::MAX_LEN);
		let suffix = format!(
			"{}{:04x}.{unit_type}",
			unit_type.unit_name_suffix_separator(),
			rand::rng().random::<u16>()
		);
		let mut max_len = UnitName::MAX_LEN - PREFIX.len() - suffix.len() - SEPARATOR.len();

		ret.push_str(PREFIX);
		if let Some(launcher) = launcher {
			max_len -= SEPARATOR.len();
			let launcher = launcher.into();
			let launcher = launcher.truncate(max_len / 2);
			max_len -= launcher.len();
			write!(ret, "{SEPARATOR}{launcher}").unwrap_or_else(|_| unreachable!());
		}
		write!(
			ret,
			"{SEPARATOR}{}{suffix}",
			app_name.into().truncate(max_len)
		)
		.unwrap_or_else(|_| unreachable!());

		debug_assert!(ret.len() <= Self::MAX_LEN, "systemd unit name is too long");
		Self(ret.into())
	}
}
impl<'s> TryFrom<Str<'s>> for UnitName<'s> {
	type Error = Report;

	fn try_from(s: Str<'s>) -> Result<Self, Self::Error> {
		if s.is_empty() {
			bail!("String is empty");
		}
		if s.len() > Self::MAX_LEN {
			bail!(
				"String is too long ({}) to be a systemd unit name ({} max)",
				s.len(),
				Self::MAX_LEN
			);
		}
		for b in s.bytes() {
			if !str::is_valid_unit_name_char(b) {
				bail!(
					"{:?} is not a valid systemd unit name character",
					char::from(b)
				);
			}
		}
		Ok(Self(s))
	}
}
impl<'s> TryFrom<&'s str> for UnitName<'s> {
	type Error = <Self as TryFrom<Str<'s>>>::Error;

	fn try_from(s: &'s str) -> Result<Self, Self::Error> {
		Self::try_from(Str::from(s))
	}
}
impl TryFrom<String> for UnitName<'static> {
	type Error = <Self as TryFrom<Str<'static>>>::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> {
		Self::try_from(Str::from(s))
	}
}
impl<'s> From<UnitName<'s>> for Str<'s> {
	#[inline]
	fn from(UnitName(s): UnitName<'s>) -> Self {
		s
	}
}
impl Debug for UnitName<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}
impl Display for UnitName<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

/// Types of systemd transient units that manage processes
#[expect(dead_code, reason = "enum exhaustiveness")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub(crate) enum UnitType {
	/// [Service](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html)
	Service,
	/// [Scope](https://www.freedesktop.org/software/systemd/man/latest/systemd.scope.html#)
	Scope,
}
impl UnitType {
	/// Returns the separator used before the suffix of a unit name.
	const fn unit_name_suffix_separator(self) -> char {
		match self {
			Self::Service => '@',
			Self::Scope => '-',
		}
	}
}
impl Display for UnitType {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		self.serialize(f)
	}
}
