//! Presets shipped with GC-Forge.
//!
//! Iteration 1 (bootstrap): placeholder only. Real presets are embedded
//! starting iteration 5 (`run-end-to-end`) via `build.rs` + `include_str!`.

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
