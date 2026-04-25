//! Loading scenarios from disk.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, ScenarioError};
use crate::scenario::{Scenario, API_VERSION, KIND};

impl Scenario {
    /// Loads a scenario from disk.
    ///
    /// Performs the YAML parse and the `apiVersion`/`kind` check. Does not
    /// resolve `extends:` (use [`Scenario::resolve_extends`] for that).
    ///
    /// # Errors
    /// Fails on I/O, YAML, or apiVersion/kind mismatch.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let body = fs::read_to_string(path).map_err(|e| ScenarioError::Io {
            path: path.to_owned(),
            source: e,
        })?;
        Self::from_yaml(&body, path)
    }

    /// Loads a scenario from a YAML string. The `path` argument is used only
    /// to enrich error messages and to resolve relative `extends:` paths.
    ///
    /// # Errors
    /// Fails on YAML or apiVersion/kind mismatch.
    pub fn from_yaml(body: &str, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let scenario: Scenario = serde_yaml::from_str(body).map_err(|e| ScenarioError::Yaml {
            path: path.to_owned(),
            source: e,
        })?;
        scenario.check_api_version()?;
        Ok(scenario)
    }

    fn check_api_version(&self) -> Result<()> {
        if self.api_version != API_VERSION {
            return Err(ScenarioError::UnsupportedApiVersion {
                found: self.api_version.clone(),
                expected: API_VERSION,
            });
        }
        if self.kind != KIND {
            return Err(ScenarioError::UnsupportedKind {
                found: self.kind.clone(),
                expected: KIND,
            });
        }
        Ok(())
    }
}

/// Resolves a relative `extends:` path against the directory containing the
/// referencing scenario.
pub(crate) fn resolve_extends_path(referrer: &Path, target: &Path) -> PathBuf {
    if target.is_absolute() {
        target.to_owned()
    } else {
        referrer
            .parent()
            .map_or_else(|| target.to_owned(), |dir| dir.join(target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_scenario(yaml: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        f
    }

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

    #[test]
    fn loads_a_good_scenario() {
        let f = write_scenario(good_yaml());
        let s = Scenario::from_path(f.path()).unwrap();
        assert_eq!(s.metadata.name, "smoke");
    }

    #[test]
    fn rejects_unknown_api_version() {
        let bad = good_yaml().replace("gc-forge/scenario.v1", "gc-forge/scenario.v2");
        let f = write_scenario(&bad);
        let err = Scenario::from_path(f.path()).unwrap_err();
        assert!(matches!(err, ScenarioError::UnsupportedApiVersion { .. }));
    }

    #[test]
    fn rejects_unknown_kind() {
        let bad = good_yaml().replace("kind: Scenario", "kind: Bogus");
        let f = write_scenario(&bad);
        let err = Scenario::from_path(f.path()).unwrap_err();
        assert!(matches!(err, ScenarioError::UnsupportedKind { .. }));
    }

    #[test]
    fn yaml_error_carries_the_path() {
        let bad = "not even yaml: : :";
        let f = write_scenario(bad);
        let err = Scenario::from_path(f.path()).unwrap_err();
        assert!(matches!(err, ScenarioError::Yaml { .. }));
    }

    #[test]
    fn missing_file_is_io_error() {
        let err = Scenario::from_path("/nope/does/not/exist.yaml").unwrap_err();
        assert!(matches!(err, ScenarioError::Io { .. }));
    }

    #[test]
    fn extends_path_resolves_relative_to_referrer() {
        let referrer = Path::new("/tmp/scenarios/derived.yaml");
        let target = Path::new("base.yaml");
        assert_eq!(
            resolve_extends_path(referrer, target),
            PathBuf::from("/tmp/scenarios/base.yaml")
        );
    }

    #[test]
    fn extends_path_keeps_absolute() {
        let referrer = Path::new("/tmp/scenarios/derived.yaml");
        let target = Path::new("/etc/some/base.yaml");
        assert_eq!(resolve_extends_path(referrer, target), target);
    }
}
