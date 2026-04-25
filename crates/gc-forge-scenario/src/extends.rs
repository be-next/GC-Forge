//! Resolution of the `extends:` chain.
//!
//! Strategy: load each scenario in the chain as a free-form `serde_yaml::Value`
//! tree, deep-merge children onto parents (children win on scalars, maps merge
//! recursively, sequences are replaced wholesale), then deserialise the final
//! merged tree into a typed [`Scenario`]. This keeps the merge logic
//! independent of the typed schema and keeps the typed structures
//! `deny_unknown_fields`-strict.
//!
//! Cycles are detected by canonicalising paths.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

use crate::error::{Result, ScenarioError};
use crate::loader::resolve_extends_path;
use crate::scenario::{Scenario, API_VERSION, KIND};

impl Scenario {
    /// Resolves the `extends:` chain rooted at `path` and returns the fully
    /// merged scenario.
    ///
    /// # Errors
    /// Fails on I/O, YAML parse error, missing apiVersion/kind, or a cycle.
    pub fn resolve(path: impl AsRef<Path>) -> Result<Self> {
        let merged = load_and_merge(path.as_ref(), &mut HashSet::new())?;
        // Strip the `extends` field from the merged tree before typed
        // deserialisation: it is a load-time directive, not part of the
        // typed `Scenario` schema as far as downstream consumers care.
        let mut merged = merged;
        if let Value::Mapping(map) = &mut merged {
            map.remove(Value::String("extends".to_owned()));
        }
        // Re-attach apiVersion/kind sanity check via from_yaml.
        let yaml = serde_yaml::to_string(&merged).map_err(|e| ScenarioError::Yaml {
            path: path.as_ref().to_owned(),
            source: e,
        })?;
        Self::from_yaml(&yaml, path)
    }
}

fn load_and_merge(path: &Path, seen: &mut HashSet<PathBuf>) -> Result<Value> {
    let canonical = fs::canonicalize(path).map_err(|e| ScenarioError::Io {
        path: path.to_owned(),
        source: e,
    })?;
    if !seen.insert(canonical.clone()) {
        return Err(ScenarioError::ExtendsCycle { path: canonical });
    }

    let body = fs::read_to_string(&canonical).map_err(|e| ScenarioError::Io {
        path: canonical.clone(),
        source: e,
    })?;
    let mut tree: Value = serde_yaml::from_str(&body).map_err(|e| ScenarioError::Yaml {
        path: canonical.clone(),
        source: e,
    })?;

    // Sanity-check the descendant's apiVersion/kind eagerly so that we catch
    // mismatches at load time rather than after a confusing merge.
    check_api_version_kind(&tree, &canonical)?;

    if let Some(extends) = take_extends(&mut tree) {
        let parent_path = resolve_extends_path(&canonical, &extends);
        let parent = load_and_merge(&parent_path, seen)?;
        let merged = deep_merge(parent, tree);
        Ok(merged)
    } else {
        Ok(tree)
    }
}

fn take_extends(tree: &mut Value) -> Option<PathBuf> {
    let Value::Mapping(map) = tree else {
        return None;
    };
    let key = Value::String("extends".to_owned());
    match map.remove(&key)? {
        Value::String(s) => Some(PathBuf::from(s)),
        _ => None,
    }
}

fn check_api_version_kind(tree: &Value, path: &Path) -> Result<()> {
    let Value::Mapping(map) = tree else {
        return Ok(()); // typed deser will fail with a better message
    };
    if let Some(Value::String(api)) = map.get(Value::String("apiVersion".to_owned())) {
        if api != API_VERSION {
            return Err(ScenarioError::UnsupportedApiVersion {
                found: api.clone(),
                expected: API_VERSION,
            });
        }
    }
    if let Some(Value::String(kind)) = map.get(Value::String("kind".to_owned())) {
        if kind != KIND {
            return Err(ScenarioError::UnsupportedKind {
                found: kind.clone(),
                expected: KIND,
            });
        }
    }
    let _ = path; // currently unused, but kept for future error enrichment
    Ok(())
}

/// Deep-merge `child` onto `parent`. Child values win for scalars and
/// sequences; mappings are merged recursively.
fn deep_merge(parent: Value, child: Value) -> Value {
    match (parent, child) {
        (Value::Mapping(mut p), Value::Mapping(c)) => {
            for (k, cv) in c {
                let merged = match p.remove(&k) {
                    Some(pv) => deep_merge(pv, cv),
                    None => cv,
                };
                p.insert(k, merged);
            }
            Value::Mapping(p)
        }
        // Otherwise child wins outright.
        (_, c) => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write(dir: &TempDir, name: &str, body: &str) -> PathBuf {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        path
    }

    fn base() -> &'static str {
        r"apiVersion: gc-forge/scenario.v1
kind: Scenario
metadata:
  name: base
spec:
  jvm: { vendor: temurin, major: 21 }
  gc:
    algorithm: G1
    options:
      heap: { min: 1g, max: 2g }
  regime:
    kind: steady-state-healthy
  duration: 30s
  seed: 0xC0FFEE
"
    }

    #[test]
    fn deep_merge_replaces_scalars_and_merges_maps() {
        let p = serde_yaml::from_str::<Value>(
            r"a:
  b: 1
  c: 2
d: keep
",
        )
        .unwrap();
        let c = serde_yaml::from_str::<Value>(
            r"a:
  c: 99
  e: 3
",
        )
        .unwrap();
        let m = deep_merge(p, c);
        let expected = serde_yaml::from_str::<Value>(
            r"a:
  b: 1
  c: 99
  e: 3
d: keep
",
        )
        .unwrap();
        assert_eq!(m, expected);
    }

    #[test]
    fn deep_merge_replaces_sequences_wholesale() {
        let p = serde_yaml::from_str::<Value>("a: [1, 2, 3]\n").unwrap();
        let c = serde_yaml::from_str::<Value>("a: [9]\n").unwrap();
        let m = deep_merge(p, c);
        assert_eq!(m, serde_yaml::from_str::<Value>("a: [9]\n").unwrap());
    }

    #[test]
    fn extends_overlays_child_onto_parent() {
        let dir = TempDir::new().unwrap();
        write(&dir, "base.yaml", base());
        let derived_path = write(
            &dir,
            "derived.yaml",
            r"apiVersion: gc-forge/scenario.v1
kind: Scenario
extends: base.yaml
metadata:
  name: derived
spec:
  gc:
    options:
      heap: { min: 2g, max: 4g }
",
        );

        let s = Scenario::resolve(&derived_path).unwrap();
        assert_eq!(s.metadata.name, "derived");
        assert_eq!(s.spec.jvm.major, 21); // inherited
        assert_eq!(
            s.spec.gc.options.heap.max.as_u64(),
            4 * crate::byte_size::ByteSize::GIB
        );
        assert_eq!(s.spec.regime.kind, "steady-state-healthy");
    }

    #[test]
    fn extends_chain_three_levels() {
        let dir = TempDir::new().unwrap();
        write(&dir, "base.yaml", base());
        write(
            &dir,
            "middle.yaml",
            r"apiVersion: gc-forge/scenario.v1
kind: Scenario
extends: base.yaml
metadata:
  name: middle
spec:
  duration: 90s
",
        );
        let leaf_path = write(
            &dir,
            "leaf.yaml",
            r"apiVersion: gc-forge/scenario.v1
kind: Scenario
extends: middle.yaml
metadata:
  name: leaf
spec:
  regime:
    kind: humongous-pressure
",
        );

        let s = Scenario::resolve(&leaf_path).unwrap();
        assert_eq!(s.metadata.name, "leaf");
        assert_eq!(s.spec.duration, crate::duration::Duration::from_secs(90));
        assert_eq!(s.spec.regime.kind, "humongous-pressure");
    }

    #[test]
    fn extends_cycle_is_detected() {
        let dir = TempDir::new().unwrap();
        let a_path = dir.path().join("a.yaml");
        let b_path = dir.path().join("b.yaml");
        std::fs::write(
            &a_path,
            r"apiVersion: gc-forge/scenario.v1
kind: Scenario
extends: b.yaml
",
        )
        .unwrap();
        std::fs::write(
            &b_path,
            r"apiVersion: gc-forge/scenario.v1
kind: Scenario
extends: a.yaml
",
        )
        .unwrap();
        let err = Scenario::resolve(&a_path).unwrap_err();
        assert!(matches!(err, ScenarioError::ExtendsCycle { .. }));
    }
}
