//! Scenario parsing and validation for GC-Forge.
//!
//! Loads YAML scenario documents conforming to `gc-forge/scenario.v1`,
//! resolves `extends:` chains, applies CLI-style overrides, and exposes a
//! JSON Schema for downstream tooling.
//!
//! # Quick example
//!
//! ```no_run
//! use gc_forge_scenario::{Override, Scenario};
//!
//! let s = Scenario::resolve("scenarios/humongous-pressure-g1.yaml")?;
//! let s = s.apply_overrides(&[Override::parse("spec.gc.options.heap.max=4g")?])?;
//! println!("name = {}", s.metadata.name);
//! # Ok::<_, gc_forge_scenario::ScenarioError>(())
//! ```

pub mod byte_size;
pub mod duration;
pub mod error;
mod extends;
mod loader;
pub mod manifest;
pub mod overrides;
pub mod scenario;
pub mod schema;

pub use byte_size::{ByteSize, ParseByteSizeError};
pub use duration::{Duration, ParseDurationError};
pub use error::{Result, ScenarioError};
pub use manifest::{
    render_manifest_schema, sha256_hex, sha256_hex_bytes, ExitStatusRecord,
    ExpectedInvariantRecord, HostMeta, JvmRecord, OutputRecord, ReproducibilityRecord, RunManifest,
    RunMeta, ScenarioRecord, ValidationRecord, ValidationResult, ValidationStatus,
    MANIFEST_API_VERSION, MANIFEST_KIND,
};
pub use overrides::Override;
pub use scenario::{
    Distribution, ExpectedClause, GcAlgorithm, GcOptions, GcSpec, HeapConfig, InvariantRule,
    JvmSpec, JvmVendor, LogFormat, Metadata, OutputSpec, RegimeSpec, Scenario, Seed, Spec,
    API_VERSION, KIND,
};
pub use schema::render_schema;

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
