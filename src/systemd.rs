//! Interaction with systemd over D-Bus

mod unit_name;

use std::fmt::{self, Debug, Display, Formatter};

use rootcause::prelude::*;
use serde::{Deserialize, Serialize};
use zbus::zvariant::{Fd, ObjectPath, OwnedValue, Str, Type, Value};

pub(crate) use self::unit_name::*;

/// Creates and starts a unit.
///
/// # Errors
/// Returns an error if:
/// - an interaction with D-Bus fails;
/// - a systemd method returns an error;
/// - systemd never signals the end of the job.
pub(super) fn start_unit(name: &UnitName, properties: &UnitProperties) -> rootcause::Result<()> {
	use std::time::Duration;

	let conn = zbus::blocking::connection::Builder::session()
		.context("Failed to prepare a D-Bus connection")?
		.method_timeout(Duration::from_millis(100))
		.build()
		.context("Failed to open a D-Bus connection")?;
	let proxy = ManagerProxy::new(&conn)
		.context("Failed to create a D-Bus object proxy")
		.attach(
			<ManagerProxy as zbus::proxy::Defaults>::INTERFACE
				.as_ref()
				.unwrap_or_else(|| unreachable!()),
		)?;

	let job_removed_stream = proxy
		.receive_job_removed()
		.context("Failed to subscribe to a D-Bus signal")
		.attach("JobRemoved()")?;

	log::debug!("Starting systemd unit:\n# {name}\n{properties}");
	let job = proxy
		.start_transient_unit(name, UnitStartMode::Fail, properties, UnitAux::new())
		.context("D-Bus method returned an error")
		.attach("StartTransientUnit()")?;
	log::trace!("Started systemd job {}", job.inner().path());

	// Wait for the job to end
	for msg in job_removed_stream {
		let job_removed = msg
			.args()
			.context("Failed to parse D-Bus signal message")
			.attach("JobRemoved()")?;
		log::trace!(
			"systemd job {} ended: {:?}",
			job_removed.job_path,
			job_removed.result
		);
		if job_removed.job_path == *job.inner().path() {
			match job_removed.result {
				JobResult::Done => return Ok(()),
				result => {
					return Err(report!("systemd job result: {result:?}")
						.attach(job.inner().path().to_string()));
				}
			}
		}
	}
	Err(report!("Timed out waiting for the systemd job to end")
		.attach(job.inner().path().to_string()))
}

/// Main entrypoint object
///
/// **Reference:** [`Manager`](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html#The%20Manager%20Object)
#[zbus::proxy(
	interface = "org.freedesktop.systemd1.Manager",
	default_service = "org.freedesktop.systemd1",
	default_path = "/org/freedesktop/systemd1",
	gen_async = false
)]
trait Manager {
	/// Creates and starts a transient unit, returning the unit starting [job](JobProxy).
	///
	/// As documented in [`StartUnit()`](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html#StartUnit()):
	/// > Callers that want to track the outcome of the actual start operation need to monitor the result of this job.
	/// > This can be achieved in a race-free manner by first [subscribing to the `JobRemoved()`](Self::receive_job_removed()) signal,
	/// > then calling `StartUnit()` and using the returned job object to filter out unrelated `JobRemoved()` signals,
	/// > until the desired one is received, which will then carry the result of the start operation.
	///
	/// **Reference:** [`StartTransientUnit()`](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html#StartTransientUnit())
	#[zbus(object = "Job", blocking_object = "JobProxy", no_autostart)]
	fn start_transient_unit(
		&self,
		name: &UnitName,
		mode: UnitStartMode,
		properties: &UnitProperties,
		aux: UnitAux,
	);

	/// Signal sent each time a job ends
	///
	/// **Reference:** [`JobRemoved()`](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html#JobRemoved())
	#[zbus(signal, no_autostart)]
	fn job_removed(
		job_id: u32,
		job_path: ObjectPath<'s>,
		unit_name: UnitName<'s>,
		result: JobResult,
	) -> zbus::Result<()>;
}

/// Scheduled or running job
///
/// **Reference:** [`Job`](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html#Job%20Objects)
#[zbus::proxy(
	interface = "org.freedesktop.systemd1.Job",
	default_service = "org.freedesktop.systemd1",
	gen_async = false
)]
trait Job {}

/// Properties of a unit
#[derive(Debug, Default, Clone, Serialize, Type)]
#[serde(transparent)]
pub(crate) struct UnitProperties<'v>(Vec<(Str<'static>, Value<'v>)>);
impl<'v> UnitProperties<'v> {
	/// Constructs a new, empty set of unit properties with at least the specified capacity.
	///
	/// See [`Vec::with_capacity()`] for more details.
	pub(crate) fn with_capacity(capacity: usize) -> Self {
		Self(Vec::with_capacity(capacity))
	}

	/// Sets the [`Slice`](https://www.freedesktop.org/software/systemd/man/latest/systemd.resource-control.html#Slice=) property.
	pub(crate) fn slice(&mut self, slice: UnitName<'v>) {
		const KEY: Str = Str::from_static("Slice");
		self.0.push((KEY, slice.into()));
	}

	/// Adds an [`After`](https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html#Before=) unit.
	pub(crate) fn add_after(&mut self, after: UnitName<'v>) {
		const KEY: Str = Str::from_static("After");
		self.0.push((KEY, vec![after].into()));
	}

	/// Adds a [`PartOf`](https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html#PartOf=) unit.
	pub(crate) fn add_part_of(&mut self, part_of: UnitName<'v>) {
		const KEY: Str = Str::from_static("PartOf");
		self.0.push((KEY, vec![part_of].into()));
	}

	/// Sets the [`CollectMode`](https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html#CollectMode=) property.
	pub(crate) fn collect_mode(&mut self, collect_mode: UnitCollectMode) {
		const KEY: Str = Str::from_static("CollectMode");
		self.0.push((KEY, collect_mode.into()));
	}

	/// Adds the PIDFD of a process that will be managed by the unit.
	///
	/// This is an undocumented property used in:
	/// - [the reference implementation `systemd-run`](https://github.com/systemd/systemd/blob/247ef81b6f2c0f92d931f5696f8d7c6fb7094dac/src/run/run.c#L1691)
	/// - [`runapp`](https://github.com/c4rlo/runapp/blob/4c0a85fbca9b23cef7b9e0d2540ce6f6b151a28f/src/main.cpp#L184)
	pub(crate) fn add_pidfd<V>(&mut self, pidfd: V)
	where
		V: Into<Fd<'v>>,
	{
		const KEY: Str = Str::from_static("PIDFDs");
		self.0.push((KEY, vec![pidfd.into()].into()));
	}
}
impl Display for UnitProperties<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		/// Displays the given [`Value`].
		///
		/// # Errors
		/// Returns an error if writing to the [`Formatter`] fails.
		fn display_value(value: &Value, f: &mut Formatter<'_>) -> fmt::Result {
			match *value {
				Value::Str(ref s) => Display::fmt(s, f),
				Value::Fd(ref fd) => Display::fmt(fd, f),
				Value::Array(ref arr) => {
					let mut first = true;
					for value in arr.inner() {
						if !std::mem::take(&mut first) {
							write!(f, " ")?;
						}
						display_value(value, f)?;
					}
					Ok(())
				}
				ref value => unimplemented!("display_value({value:?})"),
			}
		}

		let mut first = true;
		for &(ref key, ref value) in &self.0 {
			if !std::mem::take(&mut first) {
				writeln!(f)?;
			}
			write!(f, "{key}=")?;
			display_value(value, f)?;
		}
		Ok(())
	}
}

/// Unit garbage collection algorithm
///
/// **Reference:** [`CollectMode`](https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html#CollectMode=)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Type, Value, OwnedValue)]
#[serde(rename_all = "kebab-case")]
#[zvariant(signature = "s", rename_all = "kebab-case")]
#[non_exhaustive]
pub(super) enum UnitCollectMode {
	/// The unit will be unloaded of it is in the `inactive` state and is not referenced by clients, jobs or other units.
	///
	/// However it is not unloaded if it is in the `failed` state.
	/// See [`Self::InactiveOrFailed`] to unload `failed` units.
	#[default]
	Inactive,
	/// Same as [`Self::Inactive`], but the unit is unloaded even if it is in the `failed` state.
	InactiveOrFailed,
}

/// Unit starting mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Type, Value, OwnedValue)]
#[serde(rename_all = "kebab-case")]
#[zvariant(signature = "s", rename_all = "kebab-case")]
#[non_exhaustive]
enum UnitStartMode {
	/// Start the unit and its dependencies, possibly replacing already queued jobs that conflict with it.
	Replace,
	/// Start the unit and its dependencies, but will fail if this would change an already queued job.
	Fail,
	/// Start the unit and terminate all units that are not dependencies of it.
	Isolate,
	/// Start the unit but ignore all its dependencies.
	IgnoreDependencies,
	/// Start the unit but only ignore the requirement dependencies.
	IgnoreRequirements,
}

/// `aux` argument of [`StartTransientUnit()`](ManagerProxy::start_transient_unit())
#[derive(Debug, Default, Clone, Serialize, Type)]
#[serde(transparent)]
pub(crate) struct UnitAux<'v>(Vec<(Str<'v>, Vec<(Str<'v>, Value<'v>)>)>);
impl UnitAux<'_> {
	/// Constructs a new, empty `UnitAux`.
	pub(crate) const fn new() -> Self {
		Self(Vec::new())
	}
}

/// Result status of a [`Job`](JobProxy)
///
/// **Reference:** [`JobRemoved()`](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html#JobRemoved())
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Type, Value, OwnedValue)]
#[serde(rename_all = "lowercase")]
#[zvariant(signature = "s", rename_all = "lowercase")]
#[non_exhaustive]
enum JobResult {
	/// Job succesfully executed.
	Done,
	/// Job has been canceled.
	Canceled,
	/// Job timeout was reached.
	Timeout,
	/// Job failed.
	Failed,
	/// A job that this job depended on failed.
	Dependency,
	/// Job skipped because it did not apply to the unit's current state.
	Skipped,
}
