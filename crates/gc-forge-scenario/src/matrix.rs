//! `gc-forge/matrix.v1` — matrix runner schema.
//!
//! Reference: SPEC-FUNCTIONAL §9.2.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::error::{Result, ScenarioError};

/// Required `apiVersion` of a matrix document.
pub const MATRIX_API_VERSION: &str = "gc-forge/matrix.v1";
/// Required `kind` of a matrix document.
pub const MATRIX_KIND: &str = "Matrix";

/// Top-level matrix document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Matrix {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub spec: MatrixSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MatrixSpec {
    /// Path to the base scenario YAML; can be relative to the matrix file.
    pub base: PathBuf,

    /// Each entry is `<dotted.path>: [values]`. Iteration order is preserved
    /// so users can predict cell order.
    #[serde(default)]
    #[schemars(with = "BTreeMap<String, Vec<serde_json::Value>>")]
    pub axes: IndexMap<String, Vec<Value>>,

    /// Seeds to enumerate. When empty, the base scenario's seed is used once.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seeds: Vec<u64>,

    /// Cells matching any filter are excluded. A filter is a map of
    /// `<dotted.path>: <value>` — all entries must match for the filter to fire.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(with = "Vec<BTreeMap<String, serde_json::Value>>")]
    pub filters: Vec<IndexMap<String, Value>>,
}

/// One concrete cell of the matrix.
#[derive(Debug, Clone, PartialEq)]
pub struct MatrixCell {
    /// Dotted-path overrides to apply to the base scenario. Includes the seed
    /// override when `seeds:` is non-empty.
    pub overrides: Vec<(String, Value)>,
    /// The seed for this cell (if `seeds:` is non-empty), copied for
    /// downstream use.
    pub seed: Option<u64>,
}

impl Matrix {
    /// Loads a matrix from disk.
    ///
    /// # Errors
    /// Fails on I/O, YAML, or apiVersion/kind mismatch.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let body = fs::read_to_string(path).map_err(|e| ScenarioError::Io {
            path: path.to_owned(),
            source: e,
        })?;
        let m: Matrix = serde_yaml::from_str(&body).map_err(|e| ScenarioError::Yaml {
            path: path.to_owned(),
            source: e,
        })?;
        if m.api_version != MATRIX_API_VERSION {
            return Err(ScenarioError::UnsupportedApiVersion {
                found: m.api_version,
                expected: MATRIX_API_VERSION,
            });
        }
        if m.kind != MATRIX_KIND {
            return Err(ScenarioError::UnsupportedKind {
                found: m.kind,
                expected: MATRIX_KIND,
            });
        }
        Ok(m)
    }

    /// Resolves the matrix's `base` path relative to the matrix file, falling
    /// back to the matrix's own value when the matrix path has no parent.
    #[must_use]
    pub fn resolved_base_path(&self, matrix_path: &Path) -> PathBuf {
        if self.spec.base.is_absolute() {
            self.spec.base.clone()
        } else {
            matrix_path
                .parent()
                .map_or_else(|| self.spec.base.clone(), |dir| dir.join(&self.spec.base))
        }
    }

    /// Expands the matrix to its cells.
    #[must_use]
    pub fn expand(&self) -> Vec<MatrixCell> {
        let axes: Vec<(&str, &[Value])> = self
            .spec
            .axes
            .iter()
            .map(|(k, vs)| (k.as_str(), vs.as_slice()))
            .collect();
        let mut combos: Vec<Vec<(String, Value)>> = vec![Vec::new()];
        for (key, values) in &axes {
            let mut next = Vec::with_capacity(combos.len() * values.len().max(1));
            if values.is_empty() {
                continue;
            }
            for combo in &combos {
                for v in *values {
                    let mut c = combo.clone();
                    c.push(((*key).to_owned(), v.clone()));
                    next.push(c);
                }
            }
            combos = next;
        }

        let seeds: Vec<Option<u64>> = if self.spec.seeds.is_empty() {
            vec![None]
        } else {
            self.spec.seeds.iter().map(|s| Some(*s)).collect()
        };

        let mut cells: Vec<MatrixCell> = Vec::with_capacity(combos.len() * seeds.len());
        for combo in &combos {
            if filters_match(combo, &self.spec.filters) {
                continue;
            }
            for seed in &seeds {
                let mut overrides = combo.clone();
                if let Some(s) = seed {
                    overrides.push(("spec.seed".to_owned(), Value::Number((*s).into())));
                }
                cells.push(MatrixCell {
                    overrides,
                    seed: *seed,
                });
            }
        }
        cells
    }
}

fn filters_match(combo: &[(String, Value)], filters: &[IndexMap<String, Value>]) -> bool {
    if filters.is_empty() {
        return false;
    }
    'outer: for filter in filters {
        for (k, fv) in filter {
            let mut matched = false;
            for (ck, cv) in combo {
                if ck == k && cv == fv {
                    matched = true;
                    break;
                }
            }
            if !matched {
                continue 'outer;
            }
        }
        // Every filter entry matched ⇒ this cell is excluded.
        return true;
    }
    false
}

/// Renders the matrix JSON Schema (mirror of [`crate::render_schema`] and
/// [`crate::render_manifest_schema`]). The output is the third frozen wire
/// format (`gc-forge/matrix.v1`).
///
/// # Errors
/// Fails if `serde_json` cannot serialise the schema (extremely unlikely).
pub fn render_matrix_schema() -> Result<String> {
    let schema = schemars::schema_for!(Matrix);
    let pretty = serde_json::to_string_pretty(&schema)?;
    Ok(pretty + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matrix_from(yaml: &str) -> Matrix {
        let parsed: Matrix = serde_yaml::from_str(yaml).unwrap();
        parsed
    }

    #[test]
    fn parses_minimal_matrix() {
        let m = matrix_from(
            r"apiVersion: gc-forge/matrix.v1
kind: Matrix
spec:
  base: scenarios/g1.yaml
  axes:
    spec.gc.algorithm: [G1, ZGC]
",
        );
        assert_eq!(m.api_version, MATRIX_API_VERSION);
        assert_eq!(m.spec.base, PathBuf::from("scenarios/g1.yaml"));
        assert_eq!(m.spec.axes.len(), 1);
    }

    #[test]
    fn expand_yields_cartesian_product() {
        let m = matrix_from(
            r"apiVersion: gc-forge/matrix.v1
kind: Matrix
spec:
  base: scenarios/g1.yaml
  axes:
    spec.gc.algorithm: [G1, ZGC, Parallel]
    spec.jvm.major: [17, 21]
",
        );
        let cells = m.expand();
        assert_eq!(cells.len(), 6);
        // All cells should carry both axis assignments.
        for cell in &cells {
            assert_eq!(cell.overrides.len(), 2);
        }
    }

    #[test]
    fn expand_with_seeds_multiplies_cells() {
        let m = matrix_from(
            r"apiVersion: gc-forge/matrix.v1
kind: Matrix
spec:
  base: scenarios/g1.yaml
  axes:
    spec.gc.algorithm: [G1, ZGC]
  seeds: [1, 2, 3]
",
        );
        let cells = m.expand();
        assert_eq!(cells.len(), 6); // 2 × 3
                                    // Each cell should also carry the seed override.
        assert!(cells
            .iter()
            .all(|c| c.overrides.iter().any(|(k, _)| k == "spec.seed")));
        assert!(cells.iter().all(|c| c.seed.is_some()));
    }

    #[test]
    fn filters_drop_matching_cells() {
        let m = matrix_from(
            r"apiVersion: gc-forge/matrix.v1
kind: Matrix
spec:
  base: scenarios/g1.yaml
  axes:
    spec.gc.algorithm: [G1, ZGC, Parallel]
    spec.regime.kind: [steady-state-healthy, humongous-pressure]
  filters:
    - { spec.gc.algorithm: ZGC, spec.regime.kind: humongous-pressure }
",
        );
        let cells = m.expand();
        // 3 algos × 2 regimes = 6, minus 1 filtered ⇒ 5
        assert_eq!(cells.len(), 5);
        // Make sure the filtered cell is not present.
        for cell in &cells {
            let zgc = cell
                .overrides
                .iter()
                .any(|(k, v)| k == "spec.gc.algorithm" && v == &Value::String("ZGC".to_owned()));
            let humongous = cell.overrides.iter().any(|(k, v)| {
                k == "spec.regime.kind" && v == &Value::String("humongous-pressure".to_owned())
            });
            assert!(!(zgc && humongous));
        }
    }

    #[test]
    fn no_seeds_yields_one_cell_per_combo() {
        let m = matrix_from(
            r"apiVersion: gc-forge/matrix.v1
kind: Matrix
spec:
  base: scenarios/g1.yaml
  axes:
    spec.gc.algorithm: [G1, ZGC]
",
        );
        let cells = m.expand();
        assert_eq!(cells.len(), 2);
        for cell in &cells {
            assert!(cell.seed.is_none());
            assert!(!cell.overrides.iter().any(|(k, _)| k == "spec.seed"));
        }
    }

    #[test]
    fn rejects_unknown_api_version() {
        let yaml = r"apiVersion: gc-forge/matrix.v0
kind: Matrix
spec:
  base: scenarios/g1.yaml
  axes: {}
";
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("m.yaml");
        std::fs::write(&path, yaml).unwrap();
        let err = Matrix::from_path(&path).unwrap_err();
        assert!(matches!(err, ScenarioError::UnsupportedApiVersion { .. }));
    }

    #[test]
    fn matrix_schema_renders() {
        let s = render_matrix_schema().unwrap();
        assert!(s.contains("\"title\": \"Matrix\""));
    }

    /// Fails if the on-disk matrix schema has drifted from what
    /// `render_matrix_schema` produces. Regenerate with
    /// `cargo run -p gc-forge-scenario --bin gen-schema`.
    #[test]
    fn matrix_schema_matches_disk() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(|root| root.join("schemas/matrix-v1.json"))
            .expect("workspace root");
        let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "{} does not exist — run `cargo run -p gc-forge-scenario --bin gen-schema` to create it.",
                path.display()
            )
        });
        let fresh = render_matrix_schema().unwrap();
        assert!(
            on_disk == fresh,
            "matrix JSON schema drifted from {}.\n\
             Regenerate with: cargo run -p gc-forge-scenario --bin gen-schema",
            path.display()
        );
    }
}
