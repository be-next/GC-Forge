//! JVM orchestration for GC-Forge.
//!
//! Iteration 3 lands the [`Runner`] trait, the [`DockerRunner`] backend, and
//! the JVM flag builder. The native runner lands later (iter 18+ in the
//! roadmap).

pub mod docker;
pub mod flags;
pub mod runner;

pub use docker::DockerRunner;
pub use flags::{build_jvm_command, log_decorators};
pub use runner::{ExitStatus, RunOutcome, RunSpec, Runner, RunnerError};

/// Returns this crate's version.
#[must_use]
pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::version;

    #[test]
    fn version_is_non_empty() {
        assert!(!version().is_empty());
    }
}
