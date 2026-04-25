//! `--override KEY=VALUE` parsing and application.
//!
//! Strategy: parse each override as a dotted path + a YAML scalar/value;
//! apply the overrides on the YAML tree representation of the resolved
//! scenario; deserialise the result.
//!
//! Reference: SPEC-FONCTIONNELLE §5.4.

use std::path::Path;

use serde_yaml::Value;

use crate::error::{Result, ScenarioError};
use crate::scenario::Scenario;

/// A parsed override, as produced by the CLI from a `KEY=VALUE` string.
#[derive(Debug, Clone, PartialEq)]
pub struct Override {
    pub path: Vec<String>,
    pub raw_value: String,
}

impl Override {
    /// Parses a `"a.b.c=value"` form.
    ///
    /// # Errors
    /// Fails if there is no `=` separator, the key is empty, or any segment is
    /// empty.
    pub fn parse(raw: &str) -> Result<Self> {
        let (key, value) =
            raw.split_once('=')
                .ok_or_else(|| ScenarioError::InvalidOverrideSyntax {
                    raw: raw.to_owned(),
                })?;
        let key = key.trim();
        if key.is_empty() {
            return Err(ScenarioError::InvalidOverrideSyntax {
                raw: raw.to_owned(),
            });
        }
        let path: Vec<String> = key.split('.').map(str::to_owned).collect();
        if path.iter().any(String::is_empty) {
            return Err(ScenarioError::InvalidOverrideSyntax {
                raw: raw.to_owned(),
            });
        }
        Ok(Self {
            path,
            raw_value: value.to_owned(),
        })
    }
}

impl Scenario {
    /// Applies a list of overrides to a fully-resolved scenario tree.
    ///
    /// Each override is parsed as YAML for the value, allowing scalars
    /// (`"4g"`, `42`, `true`) and even YAML literals (`"[a, b]"`).
    /// The result is re-deserialised through the typed `Scenario`, so any
    /// override that breaks the schema fails loudly.
    ///
    /// # Errors
    /// Fails if an override path is missing or the resulting tree fails to
    /// re-deserialise.
    pub fn apply_overrides(self, overrides: &[Override]) -> Result<Self> {
        if overrides.is_empty() {
            return Ok(self);
        }
        let mut tree = serde_yaml::to_value(&self).map_err(|e| ScenarioError::Yaml {
            path: Path::new("<resolved>").to_owned(),
            source: e,
        })?;
        for ov in overrides {
            apply_one(&mut tree, ov)?;
        }
        let yaml = serde_yaml::to_string(&tree).map_err(|e| ScenarioError::Yaml {
            path: Path::new("<resolved>").to_owned(),
            source: e,
        })?;
        Self::from_yaml(&yaml, "<resolved>")
    }
}

fn apply_one(tree: &mut Value, ov: &Override) -> Result<()> {
    let parsed_value: Value =
        serde_yaml::from_str(&ov.raw_value).map_err(|e| ScenarioError::OverrideCoercion {
            path: ov.path.join("."),
            value: ov.raw_value.clone(),
            message: e.to_string(),
        })?;
    set_at_path(tree, &ov.path, parsed_value, &ov.path.join("."))
}

fn set_at_path(tree: &mut Value, segments: &[String], value: Value, full_path: &str) -> Result<()> {
    let Some((head, rest)) = segments.split_first() else {
        // Reached the end: replace the value at the current position.
        *tree = value;
        return Ok(());
    };

    let Value::Mapping(map) = tree else {
        return Err(ScenarioError::InvalidOverridePath {
            path: full_path.to_owned(),
        });
    };
    let key = Value::String(head.clone());
    let entry = map
        .get_mut(&key)
        .ok_or_else(|| ScenarioError::InvalidOverridePath {
            path: full_path.to_owned(),
        })?;
    set_at_path(entry, rest, value, full_path)
}

impl Override {
    /// Constructs an override directly without going through `KEY=VALUE` parsing.
    #[must_use]
    pub fn new(
        path: impl IntoIterator<Item = impl Into<String>>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into_iter().map(Into::into).collect(),
            raw_value: value.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::byte_size::ByteSize;

    fn good_yaml() -> &'static str {
        r"apiVersion: gc-forge/scenario.v1
kind: Scenario
metadata:
  name: smoke
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

    fn good() -> Scenario {
        Scenario::from_yaml(good_yaml(), "<test>").unwrap()
    }

    #[test]
    fn parses_simple_override() {
        let ov = Override::parse("spec.gc.options.heap.max=4g").unwrap();
        assert_eq!(
            ov.path,
            vec![
                "spec".to_owned(),
                "gc".to_owned(),
                "options".to_owned(),
                "heap".to_owned(),
                "max".to_owned(),
            ]
        );
        assert_eq!(ov.raw_value, "4g");
    }

    #[test]
    fn rejects_missing_equals() {
        let err = Override::parse("spec.gc.options.heap.max").unwrap_err();
        assert!(matches!(err, ScenarioError::InvalidOverrideSyntax { .. }));
    }

    #[test]
    fn rejects_empty_segments() {
        let err = Override::parse("spec..gc=foo").unwrap_err();
        assert!(matches!(err, ScenarioError::InvalidOverrideSyntax { .. }));
    }

    #[test]
    fn rejects_empty_key() {
        let err = Override::parse("=foo").unwrap_err();
        assert!(matches!(err, ScenarioError::InvalidOverrideSyntax { .. }));
    }

    #[test]
    fn applies_scalar_override() {
        let s = good();
        let ov = Override::parse("spec.gc.options.heap.max=4g").unwrap();
        let s = s.apply_overrides(&[ov]).unwrap();
        assert_eq!(s.spec.gc.options.heap.max, ByteSize(4 * ByteSize::GIB));
    }

    #[test]
    fn applies_multiple_overrides() {
        let s = good();
        let ovs = [
            Override::parse("spec.gc.options.heap.max=8g").unwrap(),
            Override::parse("spec.duration=2m").unwrap(),
        ];
        let s = s.apply_overrides(&ovs).unwrap();
        assert_eq!(s.spec.gc.options.heap.max, ByteSize(8 * ByteSize::GIB));
        assert_eq!(s.spec.duration, crate::duration::Duration::from_secs(120));
    }

    #[test]
    fn unknown_path_fails() {
        let s = good();
        let ov = Override::parse("spec.gc.bogus=42").unwrap();
        let err = s.apply_overrides(&[ov]).unwrap_err();
        assert!(matches!(err, ScenarioError::InvalidOverridePath { .. }));
    }

    #[test]
    fn override_breaking_schema_is_rejected() {
        let s = good();
        let ov = Override::parse("spec.gc.algorithm=BogusGC").unwrap();
        let err = s.apply_overrides(&[ov]).unwrap_err();
        assert!(matches!(err, ScenarioError::Yaml { .. }));
    }
}
