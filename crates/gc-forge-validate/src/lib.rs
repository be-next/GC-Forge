//! Post-run log validation against regime invariants.
//!
//! Iteration 13 lands the GC log parser, the rule evaluator, and the
//! high-level [`validate_invariants`] entry point used by `gc-forge validate`.

pub mod invariant;
pub mod parser;
pub mod validator;

pub use invariant::{evaluate, Outcome};
pub use parser::{parse_log, parse_log_text, GcEvent, GcEventKind, ParseError, ParsedLog};
pub use validator::{apply_validation, validate_invariants};

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
