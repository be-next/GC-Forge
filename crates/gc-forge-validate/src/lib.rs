//! Post-run log validation against regime invariants.
//!
//! Iteration 1 (bootstrap): placeholder only. The validator lands in
//! iteration 13 (`validate-cmd`).

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
