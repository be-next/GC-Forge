//! [`Runner`] trait, [`RunSpec`], [`RunOutcome`], and error types.
//!
//! Reference: SPEC-TECHNIQUE §4.5.

use std::path::PathBuf;
use std::time::SystemTime;

use thiserror::Error;

use gc_forge_scenario::{ByteSize, Scenario};

/// Pluggable JVM execution backend.
pub trait Runner {
    /// Human-readable name (`"docker"`, `"native"`, …).
    fn name(&self) -> &'static str;

    /// Verifies that the backend is usable on this host. Cheap check;
    /// the runner may, e.g., probe `docker version`.
    ///
    /// # Errors
    /// Returns [`RunnerError::NotAvailable`] if the backend cannot be reached
    /// (Docker daemon down, JDK not installed, …).
    fn check_available(&self) -> Result<(), RunnerError>;

    /// Runs the scenario described by `spec` to completion (or timeout).
    /// On success, returns metadata about the run; the GC log is left at
    /// `spec.log_path` for the caller to validate or post-process.
    ///
    /// # Errors
    /// Surfaces I/O, spawn, and timeout failures as [`RunnerError`] variants.
    /// A non-zero JVM exit is *not* an error: it is reported via
    /// [`RunOutcome::exit_status`].
    fn execute(&self, spec: &RunSpec) -> Result<RunOutcome, RunnerError>;
}

/// Input for a run. The scenario is already resolved (no `extends:` left)
/// and overrides have been applied.
#[derive(Debug, Clone)]
pub struct RunSpec {
    /// The fully-resolved scenario.
    pub scenario: Scenario,

    /// Where the GC log file must be written, on the *host* filesystem.
    pub log_path: PathBuf,

    /// Path to the `workload-harness.jar` on the *host* filesystem.
    pub harness_jar: PathBuf,

    /// Command-line arguments handed to the harness `main(String[])`.
    pub workload_args: Vec<String>,

    /// Optional CPU limit (passed to `docker run --cpus=…`).
    pub cpu_limit: Option<f64>,

    /// Optional memory limit (passed to `docker run --memory=…`).
    pub memory_limit: Option<ByteSize>,
}

/// Result of a successful (or partially-successful) run.
#[derive(Debug, Clone)]
pub struct RunOutcome {
    /// Where the log was written.
    pub log_path: PathBuf,

    /// Process termination status.
    pub exit_status: ExitStatus,

    /// When the JVM process started, on the host wall clock.
    pub started_at: SystemTime,

    /// When the JVM process exited.
    pub ended_at: SystemTime,

    /// Best-effort JVM identification, captured before the run via a probe.
    /// `None` if the probe was unavailable or timed out (rare).
    pub jvm_version: Option<String>,

    /// Image tag (Docker) or path to the JDK (native) the runner used.
    pub jvm_locator: String,
}

/// Process termination status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    Success,
    Failure(i32),
    /// JVM exited with the canonical OOM trampoline (exit code 3 by default
    /// or the `-XX:OnOutOfMemoryError` script exit), or its stderr matched
    /// `OutOfMemoryError`.
    Oom,
    /// The runner enforced a wall-clock timeout.
    Timeout,
    /// Killed by signal (Unix only).
    Signal(i32),
}

impl ExitStatus {
    /// Returns `true` when the run is considered successful for downstream
    /// pipelines. `Oom` is *not* a success — but it is a defined failure
    /// mode that downstream validators may explicitly tolerate.
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Success)
    }
}

/// Anything that can go wrong inside a runner.
#[derive(Debug, Error)]
pub enum RunnerError {
    /// The backend itself is not installed or not running on this host.
    #[error("runner {name:?} not available: {reason}")]
    NotAvailable { name: &'static str, reason: String },

    /// Could not spawn the JVM process.
    #[error("failed to spawn JVM process: {0}")]
    Spawn(#[source] std::io::Error),

    /// I/O failure related to the log file or stdio capture.
    #[error("log capture failure for {path:?}: {source}")]
    LogCapture {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to wait for the JVM process.
    #[error("failed to wait for JVM process: {0}")]
    Wait(#[source] std::io::Error),

    /// The runner enforced a wall-clock timeout.
    #[error("runner enforced timeout after {secs}s")]
    Timeout { secs: u64 },

    /// Catch-all for backend-specific failures.
    #[error("runner failure: {0}")]
    Other(String),
}
