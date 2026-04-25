//! Run manifest — the persistent record of one execution.
//!
//! Reference: SPEC-FONCTIONNELLE §6.2.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{Result, ScenarioError};
use crate::scenario::Scenario;

/// Required `apiVersion` of a run manifest document.
pub const MANIFEST_API_VERSION: &str = "gc-forge/run-manifest.v1";
/// Required `kind` of a run manifest document.
pub const MANIFEST_KIND: &str = "RunManifest";

/// Top-level run manifest document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunManifest {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,

    pub run: RunMeta,
    pub scenario: ScenarioRecord,
    pub jvm: JvmRecord,
    pub reproducibility: ReproducibilityRecord,
    pub output: OutputRecord,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_phenomena: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_invariants: Vec<ExpectedInvariantRecord>,

    pub validation: ValidationRecord,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunMeta {
    pub id: String,
    #[schemars(with = "String")]
    pub started_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub ended_at: DateTime<Utc>,
    pub duration_actual: crate::Duration,
    pub exit_status: ExitStatusRecord,
    pub host: HostMeta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ExitStatusRecord {
    Success,
    Failure { code: i32 },
    Oom,
    Timeout,
    Signaled { signal: i32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HostMeta {
    pub os: String,
    pub arch: String,
    pub cpu_count: u32,
    /// `docker:<image-tag>` or `native:<jdk-path>`.
    pub container: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScenarioRecord {
    pub source_path: PathBuf,
    pub source_sha256: String,
    pub resolved: Scenario,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JvmRecord {
    pub vendor: String,
    /// Human-readable version string from `java -version`, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReproducibilityRecord {
    /// The seed as a hex string, e.g. `"0xC0FFEE"`.
    pub seed: String,
    pub workload_jar_sha256: String,
    pub gc_forge_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputRecord {
    pub log_path: PathBuf,
    pub log_sha256: String,
    pub log_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectedInvariantRecord {
    pub rule: String,
    /// Free-form (number/string/array) — schema describes a generic value.
    #[serde(default)]
    #[schemars(with = "serde_json::Value")]
    pub threshold: serde_yaml::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidationRecord {
    pub status: ValidationStatus,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<ValidationResult>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub validated_at: Option<DateTime<Utc>>,

    pub validator_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidationResult {
    pub rule: String,
    #[schemars(with = "serde_json::Value")]
    pub observed: serde_yaml::Value,
    #[schemars(with = "serde_json::Value")]
    pub threshold: serde_yaml::Value,
    pub passed: bool,
}

impl RunManifest {
    /// Returns a fresh manifest with the constants `apiVersion` / `kind`
    /// pre-filled and the `validation.status` set to `Skipped`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        run: RunMeta,
        scenario: ScenarioRecord,
        jvm: JvmRecord,
        reproducibility: ReproducibilityRecord,
        output: OutputRecord,
        expected_phenomena: Vec<String>,
        expected_invariants: Vec<ExpectedInvariantRecord>,
        validator_version: String,
    ) -> Self {
        Self {
            api_version: MANIFEST_API_VERSION.to_owned(),
            kind: MANIFEST_KIND.to_owned(),
            run,
            scenario,
            jvm,
            reproducibility,
            output,
            expected_phenomena,
            expected_invariants,
            validation: ValidationRecord {
                status: ValidationStatus::Skipped,
                results: Vec::new(),
                validated_at: None,
                validator_version,
            },
        }
    }

    /// Writes the manifest to disk as YAML.
    ///
    /// # Errors
    /// Fails on serialisation or I/O.
    pub fn write_yaml(&self, path: &Path) -> Result<()> {
        let text = serde_yaml::to_string(self).map_err(|e| ScenarioError::Yaml {
            path: path.to_owned(),
            source: e,
        })?;
        write_atomic(path, text.as_bytes())
    }

    /// Writes the manifest to disk as pretty-printed JSON.
    ///
    /// # Errors
    /// Fails on serialisation or I/O.
    pub fn write_json(&self, path: &Path) -> Result<()> {
        let text = serde_json::to_string_pretty(self).map_err(ScenarioError::Schema)?;
        let mut buffer = text.into_bytes();
        buffer.push(b'\n');
        write_atomic(path, &buffer)
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| ScenarioError::Io {
                path: parent.to_owned(),
                source: e,
            })?;
        }
    }
    let tmp = path.with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp).map_err(|e| ScenarioError::Io {
            path: tmp.clone(),
            source: e,
        })?;
        f.write_all(bytes).map_err(|e| ScenarioError::Io {
            path: tmp.clone(),
            source: e,
        })?;
        f.sync_all().map_err(|e| ScenarioError::Io {
            path: tmp.clone(),
            source: e,
        })?;
    }
    fs::rename(&tmp, path).map_err(|e| ScenarioError::Io {
        path: path.to_owned(),
        source: e,
    })?;
    Ok(())
}

/// Computes the SHA-256 of a file as a hex string.
///
/// # Errors
/// Returns an [`ScenarioError::Io`] when the file cannot be read.
pub fn sha256_hex(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|e| ScenarioError::Io {
        path: path.to_owned(),
        source: e,
    })?;
    Ok(sha256_hex_bytes(&bytes))
}

/// Computes the SHA-256 of a byte slice as a hex string.
#[must_use]
pub fn sha256_hex_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut s = String::with_capacity(64);
    for b in digest {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Renders the run-manifest JSON Schema (mirror of [`crate::render_schema`]).
///
/// # Errors
/// Returns the underlying `serde_json` error if serialisation fails.
pub fn render_manifest_schema() -> Result<String> {
    let schema = schemars::schema_for!(RunManifest);
    let pretty = serde_json::to_string_pretty(&schema)?;
    Ok(pretty + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::API_VERSION;
    use chrono::TimeZone;

    fn fixture_scenario() -> Scenario {
        Scenario::from_yaml(
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
",
            "<test>",
        )
        .unwrap()
    }

    fn fixture_manifest() -> RunManifest {
        let s = fixture_scenario();
        let started = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        let ended = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 30).unwrap();
        RunManifest::new(
            RunMeta {
                id: "0190d2a0-0000-7000-8000-000000000000".to_owned(),
                started_at: started,
                ended_at: ended,
                duration_actual: crate::Duration::from_secs(30),
                exit_status: ExitStatusRecord::Success,
                host: HostMeta {
                    os: "linux".to_owned(),
                    arch: "aarch64".to_owned(),
                    cpu_count: 10,
                    container: "docker:eclipse-temurin:21-jdk-jammy".to_owned(),
                },
            },
            ScenarioRecord {
                source_path: PathBuf::from("scenarios/smoke.yaml"),
                source_sha256: sha256_hex_bytes(b"smoke source"),
                resolved: s,
            },
            JvmRecord {
                vendor: "temurin".to_owned(),
                version: Some("openjdk version \"21.0.10\"".to_owned()),
                flags: vec!["-Xms1g".to_owned(), "-Xmx2g".to_owned()],
            },
            ReproducibilityRecord {
                seed: "0xC0FFEE".to_owned(),
                workload_jar_sha256: sha256_hex_bytes(b"jar"),
                gc_forge_version: "0.0.1-dev".to_owned(),
            },
            OutputRecord {
                log_path: PathBuf::from("out/smoke-c0ffee.log"),
                log_sha256: sha256_hex_bytes(b"log content"),
                log_size_bytes: 11,
            },
            vec!["young_gc_steady".to_owned()],
            vec![],
            "iter 5".to_owned(),
        )
    }

    #[test]
    fn manifest_round_trips_yaml() {
        let m = fixture_manifest();
        let yaml = serde_yaml::to_string(&m).unwrap();
        let back: RunManifest = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn manifest_round_trips_json() {
        let m = fixture_manifest();
        let json = serde_json::to_string_pretty(&m).unwrap();
        let back: RunManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn manifest_carries_constant_api_version_and_kind() {
        let m = fixture_manifest();
        assert_eq!(m.api_version, MANIFEST_API_VERSION);
        assert_eq!(m.kind, MANIFEST_KIND);
        assert_ne!(m.api_version, API_VERSION); // not the scenario apiVersion
    }

    #[test]
    fn validation_defaults_to_skipped() {
        let m = fixture_manifest();
        assert_eq!(m.validation.status, ValidationStatus::Skipped);
        assert!(m.validation.results.is_empty());
    }

    #[test]
    fn write_yaml_then_read_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.yaml");
        let m = fixture_manifest();
        m.write_yaml(&path).unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        let back: RunManifest = serde_yaml::from_str(&body).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn sha256_hex_matches_known_vector() {
        // sha256("") == e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(
            sha256_hex_bytes(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_hex_reads_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("blob.bin");
        std::fs::write(&path, b"hello").unwrap();
        // sha256("hello") == 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        assert_eq!(
            sha256_hex(&path).unwrap(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn schema_renders() {
        let s = render_manifest_schema().unwrap();
        assert!(s.contains("\"title\": \"RunManifest\""));
    }

    #[test]
    fn exit_status_serialises_as_kind_string() {
        let yaml = serde_yaml::to_string(&ExitStatusRecord::Success).unwrap();
        assert!(yaml.contains("kind: success"), "yaml = {yaml}");
        let yaml = serde_yaml::to_string(&ExitStatusRecord::Failure { code: 7 }).unwrap();
        assert!(yaml.contains("kind: failure"), "yaml = {yaml}");
        assert!(yaml.contains("code: 7"), "yaml = {yaml}");
    }
}
