//! Application regimes for GC-Forge.
//!
//! Iteration 4 introduces the [`Regime`] trait and the first concrete
//! implementation, [`SteadyStateRegime`] (R1 in SPEC-FONCTIONNELLE §4.1).
//! The remaining six MVP regimes land in iterations 7–12.

pub mod allocation_burst;
pub mod cache_churn;
pub mod humongous_pressure;
pub mod regime;
pub mod steady_state;

pub use allocation_burst::{AllocationBurstParams, AllocationBurstRegime, BurstsCount};
pub use cache_churn::{CacheChurnParams, CacheChurnRegime};
pub use humongous_pressure::{HumongousPressureParams, HumongousPressureRegime, HumongousSize};
pub use regime::{resolve, Regime, RegimeError};
pub use steady_state::{
    LifetimeDistribution, ObjectSizeDistribution, SteadyStateParams, SteadyStateRegime,
};

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
