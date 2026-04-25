//! Application regimes for GC-Forge.
//!
//! Iteration 1 (bootstrap): placeholder only. The seven MVP regimes land
//! starting iteration 4 (`harness-steady-state`) and through iteration 12.

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
