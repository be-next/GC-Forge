//! JSON Schema generation for the scenario document.
//!
//! The schema is the source of truth for tooling outside Rust (editors, CI
//! linters, ML pipelines). We generate it from `schemars` and commit the
//! output to `schemas/scenario-v1.json`. A drift test (`schema_matches_disk`)
//! compares the on-disk file to a fresh regeneration.

use schemars::schema_for;

use crate::error::Result;
use crate::scenario::Scenario;

/// Returns the canonical, pretty-printed JSON Schema for [`Scenario`].
///
/// # Errors
/// Fails if `serde_json` cannot serialise the schema (extremely unlikely).
pub fn render_schema() -> Result<String> {
    let schema = schema_for!(Scenario);
    let pretty = serde_json::to_string_pretty(&schema)?;
    Ok(pretty + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn schema_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(|root| root.join("schemas/scenario-v1.json"))
            .expect("workspace root")
    }

    #[test]
    fn schema_renders() {
        let s = render_schema().unwrap();
        assert!(s.contains("\"title\": \"Scenario\""));
    }

    /// Fails if the on-disk scenario schema has drifted from what
    /// `render_schema` produces. Regenerate with
    /// `cargo run -p gc-forge-scenario --bin gen-schema`.
    #[test]
    fn scenario_schema_matches_disk() {
        let path = schema_path();
        let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "{} does not exist — run `cargo run -p gc-forge-scenario --bin gen-schema` to create it.",
                path.display()
            )
        });
        let fresh = render_schema().unwrap();
        assert!(
            on_disk == fresh,
            "scenario JSON schema drifted from {}.\n\
             Regenerate with: cargo run -p gc-forge-scenario --bin gen-schema",
            path.display()
        );
    }

    /// Same drift contract for the run manifest schema.
    #[test]
    fn run_manifest_schema_matches_disk() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(|root| root.join("schemas/run-manifest-v1.json"))
            .expect("workspace root");
        let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "{} does not exist — run `cargo run -p gc-forge-scenario --bin gen-schema` to create it.",
                path.display()
            )
        });
        let fresh = crate::manifest::render_manifest_schema().unwrap();
        assert!(
            on_disk == fresh,
            "manifest JSON schema drifted from {}.\n\
             Regenerate with: cargo run -p gc-forge-scenario --bin gen-schema",
            path.display()
        );
    }
}
